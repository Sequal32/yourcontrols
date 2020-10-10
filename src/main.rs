// #![windows_subsystem = "windows"]

mod app;
mod clientmanager;
mod definitions;
mod interpolate;
mod lvars;
mod server;
mod simconfig;
mod sync;
mod syncdefs;
mod varreader;
mod util;
mod update;

use app::{App, AppMessage};
use clientmanager::ClientManager;
use definitions::{Definitions, SyncPermissions};
use simconfig::Config;
use server::{Client, ReceiveData, Server, TransferClient};
use simconnect::{self, DispatchResult};
use std::{fs, io, thread, time::Duration, time::Instant};
use spin_sleep::sleep;

use sync::*;
use control::*;


const CONFIG_FILENAME: &str = "config.json";
const AIRCRAFT_DEFINITIONS_PATH: &str = "definitions/aircraft/";
const APP_STARTUP_SLEEP_TIME: Duration = Duration::from_millis(100);
const LOOP_SLEEP_TIME: Duration = Duration::from_millis(10);

fn get_aircraft_configs() -> io::Result<Vec<String>> {
    let mut filenames = Vec::new();

    for file in fs::read_dir(AIRCRAFT_DEFINITIONS_PATH)? {
        let file = file?;
        filenames.push(file.path().file_name().unwrap().to_str().unwrap().to_string())
    }

    Ok(filenames)
}

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

    let mut control = Control::new();

    let mut clients = ClientManager::new();

    // Set up sim connect
    let mut connected = false;
    let mut observing = false;

    let app_interface = App::setup();

    // Transfer
    let mut transfer_client: Option<Box<dyn TransferClient>> = None;

    //
    let mut update_rate_instant = Instant::now();
    let update_rate = 1.0 / config.update_rate as f64;
    // Whether to start a client or a server

    let mut need_update = false;
    let mut was_overloaded = false;

    let get_sync_permission = |client: &Box<dyn TransferClient>, control: &Control| -> SyncPermissions {
        if control.has_control() {
            if client.is_server() {
                SyncPermissions::ServerAndMaster
            } else {
                SyncPermissions::Master
            }
        } else if client.is_server() {
            SyncPermissions::Server
        } else {
            SyncPermissions::Slave
        }
    };

    loop {
        let timer = Instant::now();

        if let Some(client) = transfer_client.as_mut() {
            let message = conn.get_next_message();
            // Simconnect message
            match message {
                Ok(DispatchResult::SimobjectData(data)) => {
                    definitions.process_sim_object_data(data, &get_sync_permission(&client, &control));
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
            // Data from server
            match client.get_next_message() {
                Ok(ReceiveData::Update(sender, sync_data)) => {
                    if clients.is_observer(&sender) {return}
                    // need_update is used here to determine whether to sync immediately (initial connection) or to interpolate
                    definitions.on_receive_data(&conn, &sync_data, !need_update);
                    need_update = false;
                }
                Ok(ReceiveData::TransferControl(sender, to)) => {
                    // Someone is transferring controls to us
                    definitions.clear_sync();
                    if to == client.get_server_name() {
                        control.take_control(&conn);
                        app_interface.gain_control();
                        clients.set_no_control();
                    // Someone else has controls, if we have controls we let go and listen for their messages
                    } else {
                        if sender == client.get_server_name() {
                            app_interface.lose_control();
                            control.lose_control(&conn);
                        }
                        
                        app_interface.set_incontrol(&to);
                        clients.set_client_control(to);
                    }
                }
                // Increment client counter
                Ok(ReceiveData::NewConnection(name)) => {
                    if control.has_control() {
                        // Send initial aircraft state
                        client.update(definitions.get_all_current());
                    }
                    if client.is_server() {
                        app_interface.server_started(client.get_connected_count());
                    }
                    app_interface.new_connection(&name);
                    clients.add_client(name);
                },
                Ok(ReceiveData::ConnectionLost(name)) => {
                    app_interface.server_started(client.get_connected_count());
                    app_interface.lost_connection(&name);
                    clients.remove_client(&name);
                    // User may have been in control
                    if clients.client_has_control(&name) {
                        clients.set_no_control();
                        // Transfer control to myself if I'm server
                        if client.is_server() {
                            app_interface.gain_control();
                            control.has_control();
                            
                            client.transfer_control(client.get_server_name().to_string());
                        }
                    }
                }
                Ok(ReceiveData::TransferStopped(reason)) => {
                    // TAKE BACK CONTROL
                    control.take_control(&conn);
                    clients.reset();
                    observing = false;

                    match reason {
                        server::TransferStoppedReason::Requested(reason) => {
                            app_interface.client_fail(&reason);
                        }
                        server::TransferStoppedReason::ConnectionLost => {
                            app_interface.client_fail("Connection Lost.");
                        }
                    }
                }
                Ok(ReceiveData::SetObserver(target, is_observer)) => {
                    if target == client.get_server_name() {
                        observing = is_observer;
                        app_interface.observing(is_observer);
                    } else {
                        clients.set_observer(&target, is_observer);
                        app_interface.set_observing(&target, is_observer);
                    }
                }
                // Never will be reached
                Ok(ReceiveData::Name(_)) => (),
                Ok(ReceiveData::InvalidName) => {
                    client.stop("Name already in use!".to_string());
                }
                Err(_) => ()
            }

            // Handle sync vars
            if !observing && update_rate_instant.elapsed().as_secs_f64() > update_rate {
                if let Some(values) = definitions.get_need_sync(&get_sync_permission(&client, &control)) {
                    client.update(values);
                }
                update_rate_instant = Instant::now();
            }

            if !control.has_control() {
                // Message timeout
                // app_interface.update_overloaded(interpolation.overloaded());
                definitions.step_interpolate(&conn);
            }
        }

        // GUI
        match app_interface.get_next_message() {
            Ok(msg) => match msg {
                AppMessage::Server(username, is_ipv6, port ) => {
                    if connected {
                        // Display attempting to start server
                        app_interface.attempt();

                        let mut server = Server::new(username);
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
                                need_update = true;
                            },
                            Err(e) => {
                                app_interface.server_fail(e.to_string().as_str());
                                // Was error not nessecary here, stopped does not get fired
                            }
                        }
                    }
                }
                AppMessage::Connect(username, ip, input_string, port) => {
                    if connected {
                        // Display attempting to start server
                        app_interface.attempt();

                        let mut client = Client::new(username);
                        
                        match client.start(ip, port) {
                            Ok(_) => {
                                // start the client loop
                                client.run();
                                // Display connected message
                                app_interface.connected();
                                app_interface.lose_control();
                                // Assign client as the transfer client
                                transfer_client = Some(Box::new(client));
                                // Freeze aircraft
                                control.lose_control(&conn);
                                need_update = true;
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
                        client.stop("Disconnected.".to_string());
                    }
                }
                AppMessage::TransferControl(name) => {
                    if let Some(client) = transfer_client.as_ref() {
                        // Send server message
                        client.transfer_control(name.clone());
                        // Frontend who's in control
                        app_interface.set_incontrol(&name);
                        app_interface.lose_control();
                        // Log who's in control
                        clients.set_client_control(name);
                        // Freeze aircraft
                        control.lose_control(&conn);
                    }
                }
                AppMessage::SetObserver(name, is_observer) => {
                    clients.set_observer(&name, is_observer);
                    if let Some(client) = transfer_client.as_ref() {
                        client.set_observer(name, is_observer);
                    }
                }
                AppMessage::LoadAircraft(name) => {
                    definitions = Definitions::new();
                    println!("{:?}", definitions.load_config(&format!("{}{}", AIRCRAFT_DEFINITIONS_PATH, name)));
                    definitions.on_connected(&conn);
                }
                AppMessage::Startup => {
                    thread::sleep(APP_STARTUP_SLEEP_TIME);
                    app_interface.set_ip(config.ip.as_str());
                    app_interface.set_port(config.port);
                    // List aircraft
                    match get_aircraft_configs() {
                        Ok(configs) => {
                            for aircraft_config in configs {
                                app_interface.add_aircraft(&aircraft_config);
                            }
                        }
                        Err(_) => {}
                    }
                    
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
                control.on_connected(&conn);
            } else {
                // Display trying to connect message
                app_interface.error("Trying to connect to SimConnect...");
            };
            // connected = true
        }

        if timer.elapsed().as_millis() < 10 {sleep(LOOP_SLEEP_TIME)};
        // Attempt Simconnect connection
        if app_interface.exited() {break}
    }
}
