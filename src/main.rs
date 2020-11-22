#![windows_subsystem = "windows"]

mod app;
mod clientmanager;
mod definitions;
mod interpolate;
mod lvars;
mod server;
mod simconfig;
mod sync;
mod syncdefs;
mod update;
mod util;
mod varreader;

use app::{App, AppMessage, ConnectionMethod};
use clientmanager::ClientManager;
use definitions::{Definitions, SyncPermission};
use log::{error, info, warn};
use server::{Client, Event, Payloads, ReceiveMessage, Server, TransferClient};
use simconfig::Config;
use simconnect::{DispatchResult, SimConnector};
use simplelog;
use spin_sleep::sleep;
use crate::util::get_hostname_ip;
use std::{net::IpAddr, fs::{read_dir, File}, io, thread, time::Duration, time::Instant};
use update::Updater;

use control::*;
use sync::*;

const LOG_FILENAME: &str = "log.txt";
const CONFIG_FILENAME: &str = "config.json";
const AIRCRAFT_DEFINITIONS_PATH: &str = "definitions/aircraft/";

const APP_STARTUP_SLEEP_TIME: Duration = Duration::from_millis(150);
const LOOP_SLEEP_TIME: Duration = Duration::from_millis(10);

const KEY_HEVENT_PATH: &str = "definitions/resources/hevents.yaml";
const BUTTON_HEVENT_PATH: &str = "definitions/resources/touchscreenkeys.yaml";

fn get_aircraft_configs() -> io::Result<Vec<String>> {
    let mut filenames = Vec::new();

    for file in read_dir(AIRCRAFT_DEFINITIONS_PATH)? {
        let file = file?;
        filenames.push(
            file.path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        )
    }

    Ok(filenames)
}

fn write_configuration(config: &Config) {
    match config.write_to_file(CONFIG_FILENAME) {
        Ok(_) => {},
        Err(e) => error!("Could not write configuration file! Reason: {}", e)
    };
}

fn calculate_update_rate(update_rate: u16) -> f64 {1.0 / update_rate as f64}

fn start_client(timeout: u64, username: String, session_id: String, isipv6: bool, ip: Option<IpAddr>, hostname: Option<String>, port: Option<u16>, method: ConnectionMethod) -> Result<Client, String> {
    let mut client = Client::new(username, timeout);

    let client_result = match method {
        ConnectionMethod::Direct => {
            // Get either hostname ip or defined ip
            let actual_ip = match hostname {
                Some(hostname) => match get_hostname_ip(&hostname, isipv6) {
                    Ok(ip) => ip,
                    Err(e) => return Err(e.to_string())
                },
                None => ip.unwrap(),
            };

            client.start(actual_ip, port.unwrap())
        }
        ConnectionMethod::CloudServer => {
            client.start_with_hole_punch(session_id, isipv6)
        }
        ConnectionMethod::UPnP => {Ok(())}
    };

    match client_result {
        Ok(_) => Ok(client),
        Err(e) => Err(format!("Could not start client! Reason: {}", e))
    }
}

fn main() {
    // Load configuration file
    let mut config = match Config::read_from_file(CONFIG_FILENAME) {
        Ok(config) => config,
        Err(e) => {
            warn!("Could not open config. Using default values. Reason: {}", e);

            let config = Config::default();
            write_configuration(&config);
            config
        }
    };
    // Initialize logging
    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        File::create(LOG_FILENAME).unwrap(),
    )
    .ok();

    let mut conn = simconnect::SimConnector::new();

    let mut definitions = Definitions::new(config.buffer_size);
    let mut control = Control::new();
    let mut clients = ClientManager::new();

    let mut updater = Updater::new();
    let mut installer_spawned = false;

    // Set up sim connect
    let mut connected = false;
    let mut observing = false;
    // Client stopped, need to stop transfer client
    let mut should_set_none_client = false;

    let app_interface = App::setup(format!(
        "Shared Cockpit v{}", updater.get_version()
    ));

    // Transfer
    let mut transfer_client: Option<Box<dyn TransferClient>> = None;

    // Update rate counter
    let mut update_rate_instant = Instant::now();
    let mut update_rate = calculate_update_rate(config.update_rate);
    let mut last_received_update_time = 0.0;

    let mut need_update = false;

    let mut did_delay_pass = false;
    let mut init_delay_timer = Instant::now();

    let mut config_to_load = config.last_config.clone();
    // Helper closures
    // Load defintions
    let load_definitions = |conn: &SimConnector,
                            definitions: &mut Definitions,
                            config_to_load: &mut String|
     -> bool {
        // Load H Events
        match definitions.load_h_events(KEY_HEVENT_PATH, BUTTON_HEVENT_PATH) {
            Ok(_) => info!(
                "Loaded and mapped {} H: events.",
                definitions.get_number_hevents()
            ),
            Err(e) => {
                log::error!("Could not load H: event files: {}", e);
                return false;
            }
        };
        // Load aircraft configuration
        match definitions.load_config(&format!("{}{}", AIRCRAFT_DEFINITIONS_PATH, config_to_load)) {
            Ok(_) => {
                info!(
                    "Loaded and mapped {} aircraft vars, {} local vars, and {} events",
                    definitions.get_number_avars(),
                    definitions.get_number_lvars(),
                    definitions.get_number_events()
                );
                definitions.on_connected(&conn)
            }
            Err(e) => {
                log::error!(
                    "Could not load configuration file {}: {}",
                    config_to_load,
                    e
                );
                // Prevent server/client from starting as config could not be laoded.
                *config_to_load = String::new();
                return false;
            }
        };

        definitions.reset_interpolate();

        info!("{} loaded successfully.", config_to_load);

        return true;
    };

    loop {
        let timer = Instant::now();
        let message = conn.get_next_message();
        // Simconnect message
        match message {
            Ok(DispatchResult::SimobjectData(data)) => {
                definitions.process_sim_object_data(data);
            }
            // Exception occured
            Ok(DispatchResult::Exception(data)) => {
                warn!("SimConnect exception occurred: {}", unsafe {
                    data.dwException
                });
            }
            Ok(DispatchResult::Event(data)) => {
                definitions.process_event_data(data);
            }
            Ok(DispatchResult::ClientData(data)) => {
                definitions.process_client_data(&conn, data);
            }
            Ok(DispatchResult::Quit(_)) => {
                if let Some(client) = transfer_client.as_mut() {
                    client.stop("Sim closed.");
                }
            }
            _ => (),
        };

        if let Some(client) = transfer_client.as_mut() {
            if let Ok(message) = client.get_next_message() {
                match message {
                    ReceiveMessage::Payload(payload) => match payload {
                        // Unused
                        Payloads::Handshake { .. } => {}
                        Payloads::HostingReceived { .. } => {}
                        Payloads::AttemptConnection { .. } => {}
                        Payloads::PeerEstablished { .. } => {}
                        Payloads::Name{..} => {},
                        // Used
                        Payloads::Update{data, time, from} => {
                            if time > last_received_update_time {
                                    // Seamlessly transfer from losing control wihout freezing
                                if control.has_pending_transfer() {
                                    control.finalize_transfer(&conn)
                                }

                                if !clients.is_observer(&from) {
                                    definitions.on_receive_data(
                                        &conn,
                                        time,
                                        data,
                                        &SyncPermission {
                                            is_server: clients.client_is_server(&from),
                                            is_master: clients.client_has_control(&from),
                                            is_init: true,
                                        },
                                        !need_update,
                                    );
                                    // need_update is used here to determine whether to sync immediately (initial connection) or to interpolate
                                    need_update = false;
                                }
                            }
                        }
                        Payloads::TransferControl{from, to} => {
                            // Someone is transferring controls to us
                            definitions.clear_sync();
                            if to == client.get_server_name() {
                                info!("Taking control from {}", from);
                                control.take_control(&conn);
                                app_interface.gain_control();
                                clients.set_no_control();
                            // Someone else has controls, if we have controls we let go and listen for their messages
                            } else {
                                if from == client.get_server_name() {
                                    info!("Server yanked control from us.");
                                    app_interface.lose_control();
                                    control.lose_control();
                                }
                                info!("{} is now in control.", to);
                                app_interface.set_incontrol(&to);
                                clients.set_client_control(to);
                            }
                            last_received_update_time = 0.0;
                        }
                        Payloads::PlayerJoined{name, in_control, is_observer, is_server} => {
                            info!(
                                "{} connected. In control: {}, observing: {}, server: {}",
                                name, in_control, is_observer, is_server
                            );
                                // Send initial aircraft state
                            if control.has_control() {
                                client.update(definitions.get_all_current());
                            }
                            if client.is_server() {
                                app_interface.server_started(client.get_connected_count(), client.get_session_id().as_deref());
                            }

                            app_interface.new_connection(&name);
                            app_interface.set_observing(&name, is_observer);

                            clients.add_client(name.clone());
                            clients.set_server(&name, is_server);
                            clients.set_observer(&name, is_observer);
                            if in_control {
                                app_interface.set_incontrol(&name);
                                clients.set_client_control(name);
                            }
                        }
                        Payloads::PlayerLeft{name} => {
                            info!("{} lost connection.", name);
                            if client.is_server() {
                                app_interface.server_started(client.get_connected_count(), client.get_session_id().as_deref());
                            }
                            app_interface.lost_connection(&name);
                            clients.remove_client(&name);
                            // User may have been in control
                            if clients.client_has_control(&name) {
                                clients.set_no_control();
                                // Transfer control to myself if I'm server
                                if client.is_server() {
                                    info!("{} had control, taking control back.", name);
                                    app_interface.gain_control();

                                    control.take_control(&conn);
                                    client.transfer_control(client.get_server_name().to_string());
                                }
                            }
                        }
                        Payloads::SetObserver{from, to, is_observer} => {
                            if from == client.get_server_name() {
                                info!("Server set us to observing.");
                                observing = is_observer;
                                app_interface.observing(is_observer);
                            } else {
                                info!("{} is observing? {}", from, is_observer);
                                clients.set_observer(&from, is_observer);
                                app_interface.set_observing(&from, is_observer);
                            }
                        }
                        
                        Payloads::InvalidName{} => {
                            info!(
                                "{} was already in use, disconnecting.",
                                client.get_server_name()
                            );
                            client.stop("Name already in use!");
                        }
                    }
                    ReceiveMessage::Event(e) => match e {
                        Event::ConnectionEstablished => {
                            if client.is_server() {
                                    // Display server started message
                                app_interface.server_started(0, client.get_session_id().as_deref());
                                    // Unfreeze aircraft
                                control.take_control(&conn);
                                app_interface.gain_control();
                            } else {
                                    // Display connected message
                                app_interface.connected();
                                app_interface.lose_control();
                                    // Freeze aircraft
                                control.lose_control();
                            }
                            
                            last_received_update_time = 0.0;
                            need_update = true;
                            did_delay_pass = false;
                        }
                        Event::ConnectionLost(reason) => {
                            info!("Server/Client stopped. Reason: {}", reason);
                                // TAKE BACK CONTROL
                            control.take_control(&conn);

                            clients.reset();
                            observing = false;
                            should_set_none_client = true;

                            app_interface.client_fail(&reason);
                        }
                        Event::UnablePunchthrough => {
                            app_interface.client_fail("Could not connect to host! Port forwarding or hamachi is required.")
                        }
                        
                        Event::SessionIdFetchFailed => {
                            app_interface.server_fail("Could not connect to Cloud Server to fetch session ID.")
                        }
                    }
                }
            }

            // Handle sync vars
            let can_update = update_rate_instant.elapsed().as_secs_f64() > update_rate;
            // Give time for all lvars to load in
            let init_delay_passed = init_delay_timer.elapsed().as_secs() >= 3;

            if !observing && can_update && init_delay_passed {
                if did_delay_pass {

                    let permission = SyncPermission {
                        is_server: client.is_server(),
                        is_master: control.has_control(),
                        is_init: false,
                    };
    
                    if let Some(values) = definitions.get_need_sync(&permission) {
                        client.update(values);
                    }
    
                    update_rate_instant = Instant::now();

                } else {
                    did_delay_pass = true;
                    definitions.clear_need_sync();
                }
            }

            if !control.has_control() {
                definitions.step_interpolate(&conn);
            }
        }

        // GUI
        match app_interface.get_next_message() {
            Ok(msg) => match msg {
                AppMessage::StartServer {username, port, isipv6, method} => {
                    if config_to_load == "" {
                        app_interface.server_fail("Select an aircraft config first!");
                        
                    } else if !load_definitions(&conn, &mut definitions, &mut config_to_load) {

                        app_interface.error("Error loading definition files. Check the log for more information.");
                        
                    } else if connected {
                        // Display attempting to start server
                        app_interface.attempt();

                        let mut server = Box::new(Server::new(username.clone()));

                        let result = match method {
                            ConnectionMethod::Direct => {
                                server.start(isipv6, port)
                            }
                            ConnectionMethod::UPnP => {
                                server.start(isipv6, port)
                            }
                            ConnectionMethod::CloudServer => {
                                server.start_with_hole_punching(isipv6)
                            }
                        };

                        match result {
                            Ok(_) => {
                                // Assign server as transfer client
                                transfer_client = Some(server);
                                info!("Server started");
                            }
                            Err(e) => {
                                app_interface.server_fail(e.to_string().as_str());
                                info!("Could not start server! Reason: {}", e);
                            }
                        }

                        init_delay_timer = Instant::now();

                        config.port = port;
                        config.name = username;
                        write_configuration(&config);
                    }
                }
                AppMessage::Connect {session_id, username, method, ip, port, isipv6, hostname} => {
                    if config_to_load == "" {
                        app_interface.client_fail("Select an aircraft config first!");

                    } else if !load_definitions(&conn, &mut definitions, &mut config_to_load) {

                        app_interface.error("Error loading definition files. Check the log for more information.");
                        
                    } else if connected {
                        // Display attempting to start server
                        app_interface.attempt();

                        match start_client(config.conn_timeout, username.clone(), session_id, isipv6, ip, hostname, port, method) {
                            Ok(client) => {
                                info!("Client started.");
                                transfer_client = Some(Box::new(client));
                            }
                            Err(e) => {
                                app_interface.client_fail(e.to_string().as_str());
                                error!("Could not start client! Reason: {}", e);
                            }
                        }

                        init_delay_timer = Instant::now();
                        // Write config with new values
                        config.name = username;
                        config.ip = if ip.is_some() {ip.unwrap().to_string()} else {String::new()};
                        write_configuration(&config);
                    }
                }
                AppMessage::Disconnect => {
                    info!("Request to disconnect.");
                    if let Some(client) = transfer_client.as_mut() {
                        client.stop("Stopped.");
                    }
                }
                AppMessage::TransferControl {target} => {
                    if let Some(client) = transfer_client.as_ref() {
                        info!("Giving control to {}", target);
                        // Send server message
                        client.transfer_control(target.clone());
                        // Frontend who's in control
                        app_interface.set_incontrol(&target);
                        app_interface.lose_control();
                        // Log who's in control
                        clients.set_client_control(target);
                        // Freeze aircraft
                        control.lose_control();
                        // Clear interpolate
                        definitions.reset_interpolate();
                    }
                }
                AppMessage::SetObserver {target, is_observer} => {
                    clients.set_observer(&target, is_observer);
                    if let Some(client) = transfer_client.as_ref() {
                        info!("Setting {} as observer.", target);
                        client.set_observer(target, is_observer);
                    }
                }
                AppMessage::LoadAircraft {config_file_name} => {
                    // Load config
                    info!("{} aircraft config selected.", config_file_name);
                    definitions = Definitions::new(config.buffer_size);
                    config_to_load = config_file_name.clone();
                    // Clear all definitions/events/etc
                    conn.close();
                    connected = false;
                    // Save current config
                    config.last_config = config_file_name;
                    config.write_to_file(CONFIG_FILENAME).ok();
                }
                AppMessage::Startup => {
                    thread::sleep(APP_STARTUP_SLEEP_TIME);
                    // List aircraft
                    match get_aircraft_configs() {
                        Ok(configs) => {
                            info!("Found {} configuration file(s).", configs.len());

                            for aircraft_config in configs {
                                app_interface.add_aircraft(&aircraft_config);
                            }
                        }
                        Err(_) => {}
                    }
                    // Update version
                    let app_version = updater.get_version();
                    if let Ok(newest_version) = updater.get_latest_version() {
                        if *newest_version > app_version
                            && (!newest_version.is_prerelease()
                                || newest_version.is_prerelease() && config.check_for_betas)
                        {
                            app_interface.version(&newest_version.to_string());
                        }
                        info!(
                            "Version {} in use, {} is newest.",
                            app_version, newest_version
                        )
                    } else {
                        info!("Version {} in use.", app_version)
                    }
                    
                    app_interface.send_config(&config.get_json_string());
                }
                AppMessage::RunUpdater => {
                    match updater.run_installer() {
                        Ok(_) => {
                            // Terminate self
                            installer_spawned = true
                        }
                        Err(e) => {
                            error!("Downloading installer failed. Reason: {}", e);
                            app_interface.update_failed();
                        }
                    };
                }
                AppMessage::UpdateConfig {new_config} => {
                    config = new_config;
                    update_rate = calculate_update_rate(config.update_rate);
                    write_configuration(&config);
                }
            },
            Err(_) => {}
        }
        // Try to connect to simconnect if not connected
        if !connected {
            connected = conn.connect("Your Control");
            // connected = true;
            if connected {
                // Display not connected to server message
                control.on_connected(&conn);
                info!("Connected to SimConnect.");
            } else {
                // Display trying to connect message
                app_interface.error("Trying to connect to SimConnect...");
            };
        }

        if should_set_none_client {
            // Prevent sending any more data
            transfer_client = None;
            should_set_none_client = false
        }

        if timer.elapsed().as_millis() < 10 {
            sleep(LOOP_SLEEP_TIME)
        };
        // Attempt Simconnect connection
        if app_interface.exited() || installer_spawned {
            break;
        }
    }
}
