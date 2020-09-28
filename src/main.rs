// #![windows_subsystem = "windows"]

mod app;
mod definitions;
mod interpolate;
mod lvars;
mod simclient;
mod simconfig;
mod simserver;
mod sync;
mod syncdefs;
mod varreader;
mod util;

use app::{App, AppMessage};
use definitions::Definitions;
use crossbeam_channel::{Receiver, Sender};
use interpolate::InterpolateStruct;
use serde_json::{Value, json};
use simclient::Client;
use simconfig::Config;
use simconnect::{self, DispatchResult};
use simserver::{TransferClient, ReceiveData};
use simserver::Server;
use std::{io::Error, net::{IpAddr}, rc::Rc, thread, time::Duration, time::Instant};

use sync::*;
use control::*;

struct TransferStruct {
    tx: Sender<Value>,
    rx: Receiver<ReceiveData>,
    client: Box<dyn TransferClient>
}

fn start_server(is_v6: bool, port: u16) -> Result<TransferStruct, Error> {
    let mut server = Server::new();
    let (tx, rx) = server.start(is_v6, port)?;

    Ok(TransferStruct {
        tx, rx, client: Box::new(server)
    })
}

fn start_client(ip: IpAddr, port: u16) -> Result<TransferStruct, Error> {
    let client = Client::new();
    let (tx, rx) = client.start(ip, port)?;

    Ok(TransferStruct {
        tx, rx, client: Box::new(client)
    })
}

const CONFIG_FILENAME: &str = "config.json";
const APP_STARTUP_SLEEP_TIME: Duration = Duration::from_millis(100);
const LOOP_SLEEP_TIME: Duration = Duration::from_millis(1);

fn main() {
    // Load configuration file
    let mut config = match Config::read_from_file(CONFIG_FILENAME) {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            config.write_to_file(CONFIG_FILENAME).expect(format!("Could not write to {}!", CONFIG_FILENAME).as_str());
            config
        }
    };

    let mut conn = simconnect::SimConnector::new();

    let mut definitions = Definitions::new();
    definitions.load_config("aircraftdefs/C172.yaml");

    let mut control = Control::new();

    // Set up sim connect
    let mut connected = false;

    let mut app_interface = App::setup();

    // Transfer
    let mut transfer_client: Option<TransferStruct> = None;

    //
    let mut update_rate_instant = Instant::now();
    let update_rate = 1.0 / config.update_rate as f64;
    // Whether to start a client or a server

    let mut should_sync = false;
    let mut need_update = false;
    let mut was_error = false;
    let mut was_overloaded = false;
    let mut time_since_control = Instant::now();
    // Interpolation Vars //
    let mut interpolation = InterpolateStruct::new(config.buffer_size);
    interpolation.add_special_floats_regular(&mut vec!["PLANE HEADING DEGREES MAGNETIC".to_string()]);
    interpolation.add_special_floats_wrap90(&mut vec!["PLANE PITCH DEGREES".to_string()]);
    interpolation.add_special_floats_wrap180(&mut vec!["PLANE BANK DEGREES".to_string()]);

    loop {
        let mut was_no_message = true;
        if let Some(client) = transfer_client.as_mut() {
            let tx = &client.tx;
            let rx = &client.rx;

            let message = conn.get_next_message();
            // Simconnect message
            match message {
                Ok(DispatchResult::SimobjectData(data)) => {
                    definitions.process_sim_object_data(data);
                },
                // Exception occured
                Ok(DispatchResult::Exception(data)) => {
                    unsafe{println!("{:?}", data.dwException)};
                },
                Ok(DispatchResult::Event(data)) => {
                    definitions.process_event_data(data);
                },
                Ok(DispatchResult::ClientData(data)) => {
                    definitions.process_client_data(data);
                }
                _ => ()
            };

            definitions.step(&conn);
            // Data from the person in control
            match rx.try_recv() {
                Ok(ReceiveData::Data(value)) => match value["type"].as_str().unwrap() {
                    
                    "relieve_control" => {
                        control.controls_available();
                        app_interface.can_take_control();
                    },
                    "transfer_control" => {
                        if control.has_control() {
                            // Freeze aircraft
                            control.lose_control(&conn);
                            // Hide relieve control button
                            app_interface.lose_control();
                            interpolation.reset();
                            need_update = true;
                        }
                    },
                    "cancel_relieve" => {
                        control.controls_unavailable();
                        app_interface.lose_control();
                    }
                    _ => ()
                },
                Ok(ReceiveData::NewConnection(_)) | Ok(ReceiveData::ConnectionLost(_)) => {
                    app_interface.server_started(client.client.get_connected_count());
                },
                Ok(ReceiveData::TransferStopped(reason)) => {
                    app_interface.client_fail(reason.as_str());
                }
                _ => ()
            }

            // Handle sync vars
            let values = definitions.get_need_sync();
            if values.is_some() {
                println!("{:?}", values);
            }

            if !control.has_control() {
                if control.time_since_control_change().as_secs() > 10 && interpolation.get_time_since_last_position() > config.conn_timeout {
                    if !client.client.is_server() {
                        client.client.stop();
                        app_interface.client_fail("Peer timeout.");
                        was_error = true;
                    } else {
                        control.take_control(&conn);
                    }
                }

                if interpolation.overloaded() && !was_overloaded {
                    was_overloaded = true;
                    app_interface.overloaded();
                } else if !interpolation.overloaded() && was_overloaded {
                    was_overloaded = false;
                    app_interface.stable();
                }

                if !need_update {
                    if let Some(updated_map) = interpolation.interpolate() {
                        definitions.write_aircraft_data(&conn, &updated_map);
                    }
                }
            // Relieve control response timeout
            } else if control.is_relieving_control() && control.time_since_relieve().as_secs() > 20 {
                control.stop_relieiving();
                tx.try_send(json!({
                    "type": "cancel_relieve"
                })).ok();
            }

            // Server stopped or client disconnected
            if client.client.stopped() {
                control.take_control(&conn);
                transfer_client = None;
                // Message already set
                if !was_error {app_interface.disconnected()}
                was_error = false;
            }
        }

        // GUI
        match app_interface.rx.try_recv() {
            Ok(msg) => match msg {
                AppMessage::Server(is_v6, port ) => {
                    if connected {
                        app_interface.attempt();
                        match start_server(is_v6, port) {
                            Ok(transfer) => {
                                // Display server started message
                                app_interface.server_started(0);
                                // Assign server as transfer client
                                transfer_client = Some(transfer);
                                // Unfreeze aircraft
                                control.take_control(&conn);
                                app_interface.gain_control();
                            },
                            Err(e) => {
                                app_interface.server_fail(e.to_string().as_str());
                                // Was error not nessecary here, stopped does not get fired
                            }
                        }
                    }
                }
                AppMessage::Connect(ip, input_string, port) => {
                    if connected {
                        app_interface.attempt();
                        match start_client(ip, port) {
                            Ok(transfer) => {
                                // Display connected message
                                app_interface.connected();
                                // Assign client as the transfer client
                                transfer_client = Some(transfer);
                                // Freeze aircraft
                                control.lose_control(&conn);
                            }
                            Err(e) => {
                                app_interface.client_fail(e.to_string().as_str());
                                // Was error not nessecary here
                            }
                        }
                        // Write config with new values
                        config.set_ip(input_string);
                        config.write_to_file(CONFIG_FILENAME).ok();
                    }
                }
                AppMessage::Disconnect => {
                    if let Some(client) = transfer_client.as_ref() {
                        client.client.stop();
                    }
                }
                AppMessage::TakeControl => {
                    if control.take_control(&conn) {
                        app_interface.gain_control();
                        transfer_client.as_ref().unwrap().tx.try_send(json!({
                            "type": "transfer_control"
                        })).ok();
                    }
                }
                AppMessage::RelieveControl => {
                    control.relieve_control();
                    transfer_client.as_ref().unwrap().tx.try_send(json!({
                        "type": "relieve_control"
                    })).ok();
                },
                AppMessage::Startup => {
                    thread::sleep(APP_STARTUP_SLEEP_TIME);
                    app_interface.set_ip(config.ip.as_str());
                    app_interface.set_port(config.port);
                }
            }
            Err(_) => {}
        } 
        // Try to connect to simconnect if not connected
        if !connected {
            connected = conn.connect("Your Control");
            if connected {
                // Display not connected to server message
                app_interface.disconnected();
                definitions.on_connected(&conn);
                control.on_connected(&conn);
            } else {
                // Display trying to connect message
                app_interface.error("Trying to connect to SimConnect...");
            };
        }

        if was_no_message {thread::sleep(LOOP_SLEEP_TIME)}
        // Attempt Simconnect connection
        if app_interface.exited() {break}
    }
}
