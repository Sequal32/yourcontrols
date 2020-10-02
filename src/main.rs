// #![windows_subsystem = "windows"]

mod app;
mod definitions;
mod interpolate;
mod lvars;
mod server;
mod simconfig;
mod sync;
mod syncdefs;
mod varreader;
mod util;

use app::{App, AppMessage};
use definitions::Definitions;
use crossbeam_channel::{Receiver, Sender};
use interpolate::InterpolateStruct;
use serde_json::{Value, json};
use simconfig::Config;
use server::{Client, ControlTransferType, ReceiveData, Server, TransferClient};
use simconnect::{self, DispatchResult};
use std::{io::Error, net::{IpAddr}, thread, time::Duration, time::Instant};

use sync::*;
use control::*;


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
    let mut transfer_client: Option<Box<dyn TransferClient>> = None;

    //
    let mut update_rate_instant = Instant::now();
    let update_rate = 1.0 / config.update_rate as f64;
    // Whether to start a client or a server

    let mut should_sync = false;
    let mut need_update = false;
    let mut was_error = false;
    let mut was_overloaded = false;
    // Interpolation Vars //
    let mut interpolation = InterpolateStruct::new(config.buffer_size);
    interpolation.add_special_floats_regular(&mut vec!["PLANE HEADING DEGREES MAGNETIC".to_string()]);
    interpolation.add_special_floats_wrap90(&mut vec!["PLANE PITCH DEGREES".to_string()]);
    interpolation.add_special_floats_wrap180(&mut vec!["PLANE BANK DEGREES".to_string()]);

    loop {
        let mut was_no_message = true;
        if let Some(client) = transfer_client.as_mut() {
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
            match client.get_next_message() {
                Ok(ReceiveData::Update(sync_data)) => {
                    todo!()
                }
                Ok(ReceiveData::ChangeControl(control_type)) => {
                    match control_type {
                        ControlTransferType::Take => {
                            if control.has_control() {
                                // Freeze aircraft
                                control.lose_control(&conn);
                                // Hide relieve control button
                                app_interface.lose_control();
                                interpolation.reset();
                                need_update = true;
                            }
                        }
                        ControlTransferType::Relieve => {
                            control.controls_available();
                            app_interface.can_take_control();
                        }
                        ControlTransferType::Cancel => {
                            control.controls_unavailable();
                            app_interface.lose_control();
                        }
                    }
                    
                }
                // Increment client counter
                Ok(ReceiveData::NewConnection(_)) | Ok(ReceiveData::ConnectionLost(_)) => {
                    app_interface.server_started(client.get_connected_count());
                }
                Ok(ReceiveData::TransferStopped(reason)) => {
                    app_interface.client_fail(reason.as_str());
                }
                Err(_) => ()
            }

            // Handle sync vars
            let values = definitions.get_need_sync();
            if values.is_some() && control.has_control() {
                println!("{:?}", values);
                client.update(values.unwrap());
            }

            if !control.has_control() {
                // Message timeout
                // TODO: make timeout rely on server messages instead of interpolation
                if control.time_since_control_change().as_secs() > 10 && interpolation.get_time_since_last_position() > config.conn_timeout {
                    if client.is_server() {
                        control.take_control(&conn);
                    } else {
                        client.stop();
                        app_interface.client_fail("Peer timeout.");
                        was_error = true;
                    }
                }


                app_interface.update_overloaded(interpolation.overloaded());

                // Interpolate position if current position was set
                if !need_update {
                    if let Some(updated_map) = interpolation.interpolate() {
                        definitions.write_aircraft_data(&conn, &updated_map);
                    }
                }
            // Relieve control response timeout
            } else if control.is_relieving_control() && control.time_since_relieve().as_secs() > 20 {
                control.stop_relieiving();
                client.change_control(ControlTransferType::Cancel);
            }

            // Server stopped or client disconnected
            if client.stopped() {
                control.take_control(&conn);
                transfer_client = None;
                // Message already set
                if !was_error {app_interface.disconnected()}
                was_error = false;
            }
        }

        // GUI
        match app_interface.get_next_message() {
            Ok(msg) => match msg {
                AppMessage::Server(is_ipv6, port ) => {
                    if connected {
                        // Display attempting to start server
                        app_interface.attempt();

                        let mut server = Server::new();
                        match server.start(is_ipv6, port) {
                            Ok(_) => {
                                // Start the server loop
                                server.run();
                                // Display server started message
                                app_interface.server_started(0);
                                // Assign server as transfer client
                                transfer_client = Some(Box::new(server));
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
                        // Display attempting to start server
                        app_interface.attempt();

                        let mut client = Client::new();
                        
                        match client.start(ip, port) {
                            Ok(_) => {
                                // Display connected message
                                app_interface.connected();
                                // Assign client as the transfer client
                                transfer_client = Some(Box::new(client));
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
                        client.stop();
                    }
                }
                AppMessage::TakeControl => {
                    if control.take_control(&conn) {

                        if let Some(client) = transfer_client.as_ref() {
                            app_interface.gain_control();
                            client.change_control(ControlTransferType::Take)
                        }

                    }
                }
                AppMessage::RelieveControl => {
                    if let Some(client) = transfer_client.as_ref() {
                        control.relieve_control();
                        client.change_control(ControlTransferType::Relieve)
                    }
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
