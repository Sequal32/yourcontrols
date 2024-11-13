use std::{
    collections::HashMap,
    fs,
    net::IpAddr,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use simconnect::DispatchResult;
use specta::Type;
use tauri::State;
use yourcontrols_net::{Client, Event, Payloads, ReceiveMessage, Server, TransferClient};
use yourcontrols_types::AllNeedSync;

use crate::{
    client_manager::ClientManager,
    definitions::{self, ProgramAction, SyncPermission},
    simconfig::Config,
    states,
    sync::control::Control,
    update::Updater,
    util::get_hostname_ip,
    Result, AIRCRAFT_DEFINITIONS_PATH,
};

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct AircraftConfig {
    name: String,
    path: String,
}

#[tauri::command]
#[specta::specta]
pub fn get_aircraft_configs() -> Result<HashMap<String, Vec<AircraftConfig>>> {
    let mut aircraft_configs: HashMap<String, Vec<AircraftConfig>> = HashMap::new();

    let files = fs::read_dir(AIRCRAFT_DEFINITIONS_PATH)?;

    for file in files {
        let file_path = file?.path();

        // Skip non-YAML files
        if file_path.extension().is_none_or(|v| v != "yaml") {
            continue;
        }

        let file_stem = match file_path.file_stem() {
            Some(f_s) => f_s.to_string_lossy(),
            None => continue,
        };

        let config: Vec<&str> = file_stem.split(" - ").collect();

        let mut new_category = "Other";
        let mut new_aircraft_config = AircraftConfig {
            name: file_stem.to_string(),
            path: file_path.to_string_lossy().to_string(),
        };

        if config.len() == 2 {
            new_category = config[0];
            new_aircraft_config.name = config[1].to_string();
        }

        if let Some(v) = aircraft_configs.get_mut(new_category) {
            v.push(new_aircraft_config);
        } else {
            aircraft_configs.insert(new_category.to_string(), vec![new_aircraft_config]);
        }
    }

    Ok(aircraft_configs)
}

#[tauri::command]
#[specta::specta]
pub fn save_settings(
    username: String,
    aircraft: String,
    instructor_mode: bool,
    streamer_mode: bool,
) -> Result<()> {
    log::info!(
        "{} {} {} {}",
        username,
        aircraft,
        instructor_mode,
        streamer_mode
    );

    log::error!("save_settings: Not yet implemented!");

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionMethod {
    Direct,
    Relay,
    CloudServer,
}

// TODO
fn write_update_data(
    data: (Option<AllNeedSync>, Option<AllNeedSync>),
    client: &mut Box<dyn TransferClient>,
    log_sent: bool,
) {
    let (unreliable, reliable) = data;

    if let Some(data) = unreliable {
        client.update(data, true);
    }

    if let Some(data) = reliable {
        if log_sent {
            log::info!("[PACKET] SENT {:?}", data);
        }

        client.update(data, false);
    }
}

#[allow(clippy::too_many_arguments)]
fn start_client(
    timeout: u64,
    username: String,
    session_id: Option<String>,
    version: String,
    isipv6: bool,
    ip: Option<IpAddr>,
    hostname: Option<String>,
    port: Option<u16>,
    method: ConnectionMethod,
) -> std::result::Result<Client, String> {
    let mut client = Client::new(username, version, timeout);

    let client_result = match method {
        ConnectionMethod::Direct => {
            // Get either hostname ip or defined ip
            let actual_ip = match hostname {
                Some(hostname) => match get_hostname_ip(&hostname, isipv6) {
                    Ok(ip) => ip,
                    Err(e) => return Err(e.to_string()),
                },
                // If no hostname was passed, an IP must've been passed
                None => ip.unwrap(),
            };
            // A port must've been passed with direct connect
            client.start(actual_ip, port.unwrap(), session_id)
        }
        ConnectionMethod::CloudServer => client.start_with_hole_punch(session_id.unwrap(), isipv6),
        _ => panic!("Never should be reached!"),
    };

    match client_result {
        Ok(_) => Ok(client),
        Err(e) => Err(format!("Could not start client! Reason: {}", e)),
    }
}

fn connect_to_sim(
    sim_connector: &mut simconnect::SimConnector,
    definitions: &mut definitions::Definitions,
) -> anyhow::Result<()> {
    *definitions = definitions::Definitions::new();

    let connected = cfg!(feature = "skip_sim_connect") || sim_connector.connect("YourControls");
    anyhow::ensure!(
        connected,
        "Could not connect to SimConnect! Is the sim running?"
    );

    log::info!("Successfully connected to SimConnect!");
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn start_server(
    sim_connect_state: State<'_, states::SimConnectorState>,
    definitions_state: State<'_, states::DefinitionsState>,
    settings_state: State<'_, states::SettingsState>,
    transfer_client_state: State<'_, states::TransferClientState>,
    method: ConnectionMethod,
) -> Result<()> {
    {
        let mut sim_connect_mutex = sim_connect_state.lock().unwrap();
        let mut definitions_mutex = definitions_state.lock().unwrap();

        connect_to_sim(&mut sim_connect_mutex.0, &mut definitions_mutex.0)?;

        // if config_to_load.is_empty() {
        //     app_interface.server_fail("Select an aircraft config first!");
        // } else if !load_definitions(&mut definitions, &mut config_to_load) {
        //     app_interface.error(
        //         "Error loading definition files. Check the log for more information.",
        //     );
        // } else if connected {
        //     definitions.on_connected(&conn).ok();
        //     control.on_connected(&conn);
        //     // Display attempting to start server
        //     app_interface.attempt();

        log::info!("Starting server...");
        definitions_mutex.0.on_connected(&sim_connect_mutex.0)?;

        // release mutex locks
    }

    match method {
        ConnectionMethod::Relay => {
            let mut client = Box::new(Client::new(
                // username.clone(),
                "username".to_string(),
                // updater.get_version().to_string(),
                "0.0.0".to_string(),
                // config.conn_timeout,
                10,
            ));

            client.start_with_relay(false)?;

            *transfer_client_state.lock().unwrap() = Some(states::TransferClientWrapper(client));
            log::info!("Hosting started");

            // // client.start_with_relay(is_ipv6)
            // if let Err(e) = client.start_with_relay(false) {
            //     log::info!("Hosting could not start! Reason: {}", e);
            //     // TODO
            //     // app_interface.server_fail(&e.to_string());
            // } else {
            //     transfer_client = Some(client);
            //     log::info!("Hosting started");
            // }
        }
        ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
            let mut server = Box::new(Server::new(
                // username.clone(),
                "username".to_string(),
                // updater.get_version().to_string(),
                "0.0.0".to_string(),
                // config.conn_timeout,
                10,
            ));

            match method {
                // ConnectionMethod::Direct => server.start(is_ipv6, port, use_upnp),
                ConnectionMethod::Direct => server.start(false, 25071, true),
                // ConnectionMethod::CloudServer => server.start_with_hole_punching(is_ipv6),
                ConnectionMethod::CloudServer => server.start_with_hole_punching(false),
                _ => panic!("Not implemented!"),
            }?;
            // .inspect_err(|e| log::info!("Could not start server! Reason: {}", e))?;

            *transfer_client_state.lock().unwrap() = Some(states::TransferClientWrapper(server));
            log::info!("Server started");

            // if let Err(e) = result {
            //     log::info!(target: "NETWORK", "Could not start server! Reason: {}", e);
            //     // TODO
            //     // app_interface.server_fail(&e.to_string());
            // } else {
            //     // Assign server as transfer client
            //     transfer_client = Some(server);
            //     log::info!(target: "NETWORK", "Server started");
            // }
        }
    };

    let transfer_client_state = Arc::clone(&transfer_client_state);
    let sim_connect_state = Arc::clone(&sim_connect_state);
    let definitions_state = Arc::clone(&definitions_state);

    tauri::async_runtime::spawn(async move {
        // TODO
        let mut control = Control::new();
        let mut clients = ClientManager::new();
        let updater = Updater::new();
        let config = Config::default();

        // Set up sim connect
        let mut observing = false;
        // Client stopped, need to stop transfer client
        let mut should_set_none_client = false;

        let mut ready_to_process_data = false;

        let mut connection_time = None;

        // Set up loop vars
        let interval = Duration::from_millis(10);
        let mut next_time = Instant::now() + interval;

        loop {
            if let Some(client) = transfer_client_state.lock().unwrap().as_mut() {
                let client = &mut client.0;

                let conn = &sim_connect_state.lock().unwrap().0;
                let definitions = &mut definitions_state.lock().unwrap().0;

                while let Ok(message) = conn.get_next_message() {
                    match message {
                        DispatchResult::SimObjectData(data) => {
                            definitions.process_sim_object_data(data);
                        }
                        // Exception occured
                        DispatchResult::Exception(data) => {
                            log::warn!("[SIM] SimConnect exception occurred: {}", unsafe {
                                std::ptr::addr_of!(data.dwException).read_unaligned()
                            });

                            if data.dwException == 31 {
                                // Client data area was not initialized by the gauge
                                client.stop("Could not connect to the YourControls gauge. Do you have the community package installed correctly?".to_string());
                                break;
                            }
                        }
                        DispatchResult::ClientData(data) => {
                            definitions.process_client_data(data);
                        }
                        DispatchResult::Event(data) => {
                            definitions.process_event_data(data);
                        }
                        DispatchResult::Quit(_) => {
                            client.stop("Sim closed.".to_string());
                        }
                        _ => log::debug!("Message not handled: {:?}", message),
                    }
                }

                while let Ok(message) = client.get_next_message() {
                    match message {
                        ReceiveMessage::Payload(payload) => match payload {
                            // Unused
                            Payloads::Handshake { .. }
                            | Payloads::RendezvousHandshake { .. }
                            | Payloads::HostingReceived { .. }
                            | Payloads::AttemptConnection { .. }
                            | Payloads::PeerEstablished { .. }
                            | Payloads::InvalidVersion { .. }
                            | Payloads::InvalidName { .. }
                            | Payloads::RequestHosting { .. }
                            | Payloads::InitHandshake { .. }
                            | Payloads::Heartbeat => {}
                            // Used
                            Payloads::Update {
                                data,
                                from,
                                is_unreliable,
                                time,
                            } => {
                                // Not non high updating packets for debugging
                                if !is_unreliable {
                                    log::info!(
                                        "[PACKET] {:?} {} {:?} {:?} {:?}",
                                        data,
                                        from,
                                        clients.is_observer(&from),
                                        clients.client_is_server(&from),
                                        clients.client_has_control(&from)
                                    );
                                }

                                if !clients.is_observer(&from) && ready_to_process_data {
                                    match definitions.on_receive_data(
                                        conn,
                                        data,
                                        time,
                                        &SyncPermission {
                                            is_server: clients.client_is_server(&from),
                                            is_master: clients.client_has_control(&from),
                                            is_init: true,
                                        },
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            client.stop(e.to_string());
                                        }
                                    }
                                }
                            }
                            Payloads::TransferControl { from, to } => {
                                // Someone is transferring controls to us
                                definitions.reset_sync();
                                if to == client.get_server_name() {
                                    log::info!("[CONTROL] Taking control from {}", from);
                                    control.take_control(conn, &definitions.lvarstransfer.transfer);
                                    // TODO
                                    // app_interface.gain_control();
                                    clients.set_no_control();
                                // Someone else has controls, if we have controls we let go and listen for their messages
                                } else {
                                    if from == client.get_server_name() {
                                        // TODO
                                        // app_interface.lose_control();
                                        control.lose_control(
                                            conn,
                                            &definitions.lvarstransfer.transfer,
                                        );
                                    }
                                    log::info!("[CONTROL] {} is now in control.", to);
                                    // TODO
                                    // app_interface.set_incontrol(&to);
                                    clients.set_client_control(to);
                                }
                            }
                            Payloads::PlayerJoined {
                                name,
                                in_control,
                                mut is_observer,
                                is_server,
                            } => {
                                log::info!(
                                    "[NETWORK] {} connected. In control: {}, observing: {}, server: {}",
                                    name,
                                    in_control,
                                    is_observer,
                                    is_server
                                );

                                // This should be before the if statement as server_started counts the number of clients connected
                                clients.add_client(name.clone());

                                if client.is_host() {
                                    client.send_definitions(
                                        definitions.get_buffer_bytes().into_boxed_slice(),
                                        name.clone(),
                                    );

                                    if config.instructor_mode {
                                        is_observer = true;
                                        client.set_observer(name.clone(), true);
                                    }
                                }

                                // TODO
                                // app_interface.new_connection(&name);
                                // app_interface.set_observing(&name, is_observer);
                                clients.set_server(&name, is_server);
                                clients.set_observer(&name, is_observer);

                                if in_control {
                                    // TODO
                                    // app_interface.set_incontrol(&name);
                                    clients.set_client_control(name);
                                }
                            }
                            // Person is ready to receive data
                            Payloads::Ready => {
                                if control.has_control() {
                                    client.update(definitions.get_all_current(), false);
                                }
                                // Request time update to sync
                                if client.is_host() {
                                    definitions.request_time();
                                }
                            }
                            Payloads::PlayerLeft { name } => {
                                log::info!("[NETWORK] {} lost connection.", name);

                                clients.remove_client(&name);
                                // User may have been in control
                                if clients.client_has_control(&name) {
                                    clients.set_no_control();
                                    // Transfer control to myself if I'm server
                                    if client.is_host() {
                                        log::info!(
                                            "[CONTROL] {} had control, taking control back.",
                                            name
                                        );
                                        // TODO
                                        // app_interface.gain_control();

                                        control.take_control(
                                            conn,
                                            &definitions.lvarstransfer.transfer,
                                        );
                                        client
                                            .transfer_control(client.get_server_name().to_string());
                                    }
                                }

                                // TODO
                                // app_interface.lost_connection(&name);
                            }
                            Payloads::SetObserver {
                                from: _,
                                to,
                                is_observer,
                            } => {
                                if to == client.get_server_name() {
                                    log::info!(
                                        "[CONTROL] Server set us to observing? {}",
                                        is_observer
                                    );
                                    observing = is_observer;
                                    // TODO
                                    // app_interface.observing(is_observer);

                                    if !observing {
                                        definitions.reset_sync();
                                    }
                                } else {
                                    log::info!("[CONTROL] {} is observing? {}", to, is_observer);
                                    clients.set_observer(&to, is_observer);
                                    // TODO
                                    // app_interface.set_observing(&to, is_observer);
                                }
                            }
                            Payloads::SetHost => {
                                //TODO
                                // app_interface.set_host();

                                // Host was set which means successfully established connection to hoster, need to send definitions
                                client.send_definitions(
                                    definitions.get_buffer_bytes().into_boxed_slice(),
                                    client.get_server_name().to_string(),
                                );
                            }
                            Payloads::ConnectionDenied { reason } => {
                                client.stop(format!("Connection Denied: {}", reason));
                            }
                            Payloads::AircraftDefinition { bytes } => {
                                match definitions.load_config_from_bytes(bytes) {
                                    Ok(_) => {
                                        log::info!("[DEFINITIONS] Loaded and mapped {} aircraft vars, {} local vars, and {} events from the server", definitions.get_number_avars(), definitions.get_number_lvars(), definitions.get_number_events());
                                        control.on_connected(conn);

                                        let def_connect_result = definitions.on_connected(conn);
                                        if def_connect_result.is_err() {
                                            client.stop("Error starting WS server. Do you have another YourControls open?".to_string())
                                        }
                                        // Freeze aircraft
                                        control.lose_control(
                                            conn,
                                            &definitions.lvarstransfer.transfer,
                                        );
                                    }
                                    Err(e) => {
                                        log::error!("[DEFINITIONS] Could not load server sent configuration file: {}", e);
                                    }
                                }
                                // Start the connection timer to wait to send the ready payload
                                connection_time = Some(Instant::now());
                            }
                            Payloads::AttemptHosterConnection { peer } => {
                                match start_client(
                                    config.conn_timeout,
                                    client.get_server_name().to_string(),
                                    client.get_session_id(),
                                    updater.get_version().to_string(),
                                    false,
                                    Some(peer.ip()),
                                    None,
                                    Some(peer.port()),
                                    ConnectionMethod::Direct,
                                ) {
                                    Ok(new_client) => {
                                        log::info!(
                                            "[NETWORK] New client started to connect to hosted server."
                                        );
                                        *client = Box::new(new_client);
                                    }
                                    Err(e) => {
                                        // TODO
                                        // app_interface.client_fail(e.to_string().as_str());
                                        log::error!(
                                            "[NETWORK] Could not start new hoster client! Reason: {}",
                                            e
                                        );
                                    }
                                };
                            }
                            Payloads::SetSelfObserver { name } => {
                                if client.is_host() {
                                    clients.set_observer(&name, true);

                                    // TODO
                                    // app_interface.set_observing(&name, true);

                                    client.set_observer(name, true);
                                }
                            }
                        },
                        ReceiveMessage::Event(e) => match e {
                            Event::ConnectionEstablished => {
                                if client.is_host() {
                                    // Display server started message
                                    // TODO
                                    // app_interface.server_started();
                                    if let Some(session_code) = client.get_session_id().as_deref() {
                                        // TODO
                                        // app_interface.set_session_code(session_code);
                                    }
                                    // Unfreeze aircraft
                                    control.take_control(conn, &definitions.lvarstransfer.transfer);
                                    // TODO
                                    // app_interface.gain_control();
                                    // Not really used by the host
                                    connection_time = Some(Instant::now());
                                } else {
                                    // Display connected message

                                    // TODO
                                    // app_interface.connected();
                                    // app_interface.lose_control();
                                }
                            }
                            Event::ConnectionLost(reason) => {
                                log::info!("[NETWORK] Server/Client stopped. Reason: {}", reason);
                                // TAKE BACK CONTROL
                                control.take_control(conn, &definitions.lvarstransfer.transfer);

                                clients.reset();
                                observing = false;
                                should_set_none_client = true;

                                // TODO
                                // if let Err(e) = audio.play_disconnected() {
                                //     log::warn!("[AUDIO] Error playing audio: {}", e);
                                // }

                                // TODO
                                // app_interface.client_fail(&reason);
                            }
                            Event::UnablePunchthrough => {
                                // TODO
                                //     app_interface.client_fail(
                                //     "Could not connect to host! Please port forward or use 'Cloud Host'!",
                                // )
                            }

                            Event::SessionIdFetchFailed => {
                                // TODO
                                // app_interface.server_fail(
                                //     "Could not connect to Cloud Server to fetch session ID.",
                                // )
                            }

                            Event::Metrics(metrics) => {
                                // TODO
                                // app_interface.send_network(&metrics);
                            }
                        },
                    }
                }

                if let Err(e) = definitions.step(conn) {
                    client.stop(e.to_string());
                }

                // Handle specific program triggered actions
                if let Some(pending_action) = definitions.get_next_pending_action() {
                    match pending_action {
                        ProgramAction::TakeControls => {
                            if !control.has_control() && !observing {
                                if let Some(in_control) = clients.get_client_in_control() {
                                    control.take_control(conn, &definitions.lvarstransfer.transfer);
                                    client.take_control(in_control.clone());
                                }
                            }
                        }
                        ProgramAction::TransferControls => {
                            if control.has_control() {
                                if let Some(next_control) = clients.get_next_client_for_control() {
                                    client.transfer_control(next_control.clone())
                                }
                            } else if let Some(in_control) = clients.get_client_in_control() {
                                control.take_control(conn, &definitions.lvarstransfer.transfer);
                                client.take_control(in_control.clone());
                            }
                        }
                    }
                }

                // Handle initial 3 second connection delay, allows lvars to be processed
                if connection_time.is_some_and(|t| t.elapsed() >= Duration::from_secs(3)) {
                    // Do not let server send initial data - wait for data to get cleared on the previous loop
                    if !observing && ready_to_process_data {
                        let permission = SyncPermission {
                            is_server: client.is_host(),
                            is_master: control.has_control(),
                            is_init: false,
                        };

                        write_update_data(definitions.get_sync(&permission), client, true);
                    }

                    // Tell server we're ready to receive data after 3 seconds
                    if !ready_to_process_data {
                        ready_to_process_data = true;
                        definitions.reset_sync();

                        if !client.is_host() {
                            client.send_ready();
                        }
                    }
                }
            } else {
                break;
            }

            thread::sleep(next_time - Instant::now());
            next_time += interval;
        }
    });

    Ok(())
}

fn connect() {}

// AppMessage::Connect {
//     session_id,
//     username,
//     method,
//     ip,
//     port,
//     isipv6,
//     hostname,
// } => {
//     let connected = connect_to_sim(&mut conn, &mut definitions);

//     if connected {
//         // Display attempting to start server
//         app_interface.attempt();

//         match start_client(
//             config.conn_timeout,
//             username.clone(),
//             session_id,
//             updater.get_version().to_string(),
//             isipv6,
//             ip,
//             hostname,
//             port,
//             method,
//         ) {
//             Ok(client) => {
//                 info!("[NETWORK] Client started.");
//                 transfer_client = Some(Box::new(client));
//             }
//             Err(e) => {
//                 app_interface.client_fail(e.to_string().as_str());
//                 error!("[NETWORK] Could not start client! Reason: {}", e);
//             }
//         }

//         // Write config with new values
//         config.name = username;
//         config.port = port.unwrap_or(config.port);
//         config.ip = if let Some(ip) = ip {
//             ip.to_string()
//         } else {
//             String::new()
//         };
//         write_configuration(&config);
//     }
// }

#[tauri::command]
#[specta::specta]
pub fn disconnect(transfer_client_state: State<'_, states::TransferClientState>) {
    log::info!("Request to disconnect.");

    if let Some(client) = transfer_client_state.lock().unwrap().as_mut() {
        client.0.stop("Stopped.".to_string());
    }
}

#[tauri::command]
#[specta::specta]
pub fn transfer_control(
    transfer_client_state: State<'_, states::TransferClientState>,
    target: String,
) {
    if let Some(client) = transfer_client_state.lock().unwrap().as_ref() {
        log::info!("Giving control to {}", target);
        client.0.transfer_control(target);
    }
}

#[tauri::command]
#[specta::specta]
fn set_observer() {}
// AppMessage::SetObserver {
//     target,
//     is_observer,
// } => {
//     clients.set_observer(&target, is_observer);
//     if let Some(client) = transfer_client.as_ref() {
//         info!("[CONTROL] Setting {} as observer. {}", target, is_observer);
//         client.set_observer(target, is_observer);
//     }
// }

#[tauri::command]
#[specta::specta]
pub fn go_observer(transfer_client_state: State<'_, states::TransferClientState>) {
    if let Some(client) = transfer_client_state.lock().unwrap().as_ref() {
        log::info!("Request to set self as observer.");
        client.0.set_self_observer();
    }
}

fn force_take_control() {}
// AppMessage::ForceTakeControl => {
//     if let Some(client) = transfer_client.as_ref() {
//         if let Some(client_name) = clients.get_client_in_control() {
//             //Will send a loopback Payloads::TransferControl
//             client.take_control(client_name.clone())
//         }
//     }
// }

fn load_aircraft() {}
// AppMessage::LoadAircraft { config_file_name } => {
//     // Load config
//     info!(
//         "[DEFINITIONS] {} aircraft config selected.",
//         config_file_name
//     );
//     config_to_load.clone_from(&config_file_name);
// }

fn startup() {}
// AppMessage::Startup => {
//     // List aircraft
//     if let Ok(configs) = get_aircraft_configs() {
//         info!(
//             "[DEFINITIONS] Found {} configuration file(s).",
//             configs.len()
//         );

//         for aircraft_config in configs {
//             app_interface.add_aircraft(&aircraft_config);
//         }
//     }

//     app_interface.send_config(&config.get_json_string());
//     // Update version
//     let app_version = updater.get_version();
//     if let Ok(newest_version) = updater.get_latest_version() {
//         if *newest_version > app_version
//             && (newest_version.pre.is_empty()
//                 || newest_version.pre.is_empty() && config.check_for_betas)
//         {
//             app_interface.version(&newest_version.to_string());
//         }
//         info!(
//             "[UPDATER] Version {} in use, {} is newest.",
//             app_version, newest_version
//         )
//     } else {
//         info!("[UPDATER] Version {} in use.", app_version)
//     }
// }

fn run_update() {}
// AppMessage::RunUpdater => {
//     match updater.run_installer() {
//         Ok(_) => {
//             // Terminate self
//             installer_spawned = true
//         }
//         Err(e) => {
//             error!("[UPDATER] Downloading installer failed. Reason: {}", e);
//             app_interface.update_failed();
//         }
//     };
// }

fn update_config() {}
// AppMessage::UpdateConfig { new_config: config } => {
//     audio.mute(config.sound_muted);
//     write_configuration(&config);
// }
