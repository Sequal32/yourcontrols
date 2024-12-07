use std::{
    collections::HashMap,
    fs, net,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use simconnect::DispatchResult;
use specta::Type;
use tauri::State;
use tauri_specta::Event as _;
use yourcontrols_net::{Client, Event, Payloads, ReceiveMessage, Server, TransferClient};
use yourcontrols_types::AllNeedSync;

use crate::{
    definitions, events, simconfig::Config, states, sync::control::Control, update::Updater,
    util::get_hostname_ip, Result, AIRCRAFT_DEFINITIONS_PATH,
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
    ip: Option<net::IpAddr>,
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
#[allow(clippy::too_many_arguments)]
pub fn start_server(
    app_handle: tauri::AppHandle,
    sim_connect_state: State<'_, states::SimConnectorState>,
    definitions_state: State<'_, states::DefinitionsState>,
    client_manager_state: State<'_, states::ClientManagerState>,
    transfer_client_state: State<'_, states::TransferClientState>,
    method: ConnectionMethod,
    is_ipv6: bool,
    port: Option<u16>,
) -> Result<()> {
    let port = port.unwrap_or(25071);

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
    }

    // todo: maybe remove this event since the function returns a Result, the rest happens in an separate thread and needs events to communicate with the frontend
    if let Err(e) = events::server::ServerAttemptEvent.emit(&app_handle) {
        log::error!("Could not emit AttemptEvent: {:?}", e);
    }

    match method {
        ConnectionMethod::Relay => {
            let mut client = Box::new(Client::new(
                // username.clone(),
                "username".to_string(),
                // updater.get_version().to_string(),
                // TODO: version from toml?
                "2.8.5".to_string(),
                // config.conn_timeout,
                5,
            ));

            client.start_with_relay(false)?;

            *transfer_client_state.lock().unwrap() = Some(states::TransferClientWrapper(client));
            log::info!("Hosting started");
        }
        ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
            let mut server = Box::new(Server::new(
                // username.clone(),
                "username".to_string(),
                // updater.get_version().to_string(),
                // TODO: version from toml?
                "2.8.5".to_string(),
                // config.conn_timeout,
                5,
            ));

            match method {
                // ConnectionMethod::Direct => server.start(is_ipv6, port, use_upnp),
                ConnectionMethod::Direct => server.start(is_ipv6, port, true),
                ConnectionMethod::CloudServer => server.start_with_hole_punching(is_ipv6),
                _ => panic!("Not implemented!"),
            }?;

            *transfer_client_state.lock().unwrap() = Some(states::TransferClientWrapper(server));
            log::info!("Server started");
        }
    };

    let transfer_client_state = Arc::clone(&transfer_client_state);
    let sim_connect_state = Arc::clone(&sim_connect_state);
    let definitions_state = Arc::clone(&definitions_state);
    let client_manager_state = Arc::clone(&client_manager_state);

    tauri::async_runtime::spawn(async move {
        // TODO
        let mut control = Control::new();
        let updater = Updater::new();
        let config = Config::default();
        {
            client_manager_state.lock().unwrap().reset();
        }

        // Set up sim connect
        let mut observing = false;
        // Client stopped, need to stop transfer client
        let mut should_set_none_client = false;

        let mut ready_to_process_data = false;

        let mut connection_time = None;

        let mut interval = tokio::time::interval(Duration::from_millis(10));
        loop {
            interval.tick().await;

            let mut clients = client_manager_state.lock().unwrap();

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
                            log::warn!("SimConnect exception occurred: {}", unsafe {
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
                                        "{:?} {} {:?} {:?} {:?}",
                                        data,
                                        from,
                                        clients.is_observer(&from),
                                        clients.client_is_server(&from),
                                        clients.client_has_control(&from)
                                    );
                                }

                                if !clients.is_observer(&from) && ready_to_process_data {
                                    if let Err(e) = definitions.on_receive_data(
                                        conn,
                                        data,
                                        time,
                                        &definitions::SyncPermission {
                                            is_server: clients.client_is_server(&from),
                                            is_master: clients.client_has_control(&from),
                                            is_init: true,
                                        },
                                    ) {
                                        client.stop(e.to_string());
                                    }
                                }
                            }
                            Payloads::TransferControl { from, to } => {
                                // Someone is transferring controls to us
                                definitions.reset_sync();
                                if to == client.get_server_name() {
                                    log::info!("Taking control from {}", from);
                                    control.take_control(conn, &definitions.lvarstransfer.transfer);

                                    if let Err(e) = events::control::GainControlEvent.emit(&app_handle){
                                        log::error!("Could not emit GainControlEvent: {:?}", e);
                                    }

                                    clients.set_no_control();
                                // Someone else has controls, if we have controls we let go and listen for their messages
                                } else {
                                    if from == client.get_server_name() {
                                        if let Err(e) = events::control::LoseControlEvent.emit(&app_handle){
                                            log::error!("Could not emit LoseControlEvent: {:?}", e);
                                        }

                                        control.lose_control(
                                            conn,
                                            &definitions.lvarstransfer.transfer,
                                        );
                                    }
                                    log::info!("{} is now in control.", to);

                                    if let Err(e) = events::control::SetInControlEvent(to.clone()).emit(&app_handle){
                                        log::error!("Could not emit SetInControlEvent: {:?}", e);
                                    }

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
                                    "\"{}\" connected. In control: {}, observing: {}, server: {}",
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
                                clients.set_server(&name, is_server);
                                clients.set_observer(&name, is_observer);

                                if in_control {
                                    if let Err(e) = events::control::SetInControlEvent(name.clone()).emit(&app_handle){
                                        log::error!("Could not emit SetInControlEvent: {:?}", e);
                                    }

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
                                log::info!("\"{}\" lost connection.", name);

                                clients.remove_client(&name);
                                // User may have been in control
                                if clients.client_has_control(&name) {
                                    clients.set_no_control();
                                    // Transfer control to myself if I'm server
                                    if client.is_host() {
                                        log::info!("\"{}\" had control, taking control back.", name);

                                        if let Err(e) = events::control::GainControlEvent.emit(&app_handle){
                                            log::error!("Could not emit GainControlEvent: {:?}", e);
                                        }

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
                                        "Server set us to observing? {}",
                                        is_observer
                                    );
                                    observing = is_observer;
                                    // TODO
                                    // app_interface.observing(is_observer);

                                    if !observing {
                                        definitions.reset_sync();
                                    }
                                } else {
                                    log::info!("\"{}\" is observing? {}", to, is_observer);
                                    clients.set_observer(&to, is_observer);
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
                                        log::info!("Loaded and mapped {} aircraft vars, {} local vars, and {} events from the server", definitions.get_number_avars(), definitions.get_number_lvars(), definitions.get_number_events());
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
                                        log::error!("Could not load server sent configuration file: {}", e);
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
                                            "New client started to connect to hosted server."
                                        );
                                        *client = Box::new(new_client);
                                    }
                                    Err(e) => {
                                        if let Err(e) =  events::ClientFailEvent(e.to_string()).emit(&app_handle){
                                            log::error!("Could not emit ClientFailEvent: {:?}", e);
                                        }

                                        log::error!("Could not start new hoster client! Reason: {}", e);
                                    }
                                };
                            }
                            Payloads::SetSelfObserver { name } => {
                                if client.is_host() {
                                    clients.set_observer(&name, true);
                                    client.set_observer(name, true);
                                }
                            }
                            // _ => log::debug!("Payload not handled: {:?}", payload),
                            _ => {},
                        },
                        ReceiveMessage::Event(e) => match e {
                            Event::ConnectionEstablished => {
                                if client.is_host() {
                                    if let Some(session_code) = client.get_session_id() {
                                        // Display server started message
                                        if let Err(e) = events::server::ServerStartedEvent(session_code).emit(&app_handle){
                                            log::error!("Could not emit StartedEvent: {:?}", e);
                                        }
                                    }
                                    // Unfreeze aircraft
                                    control.take_control(conn, &definitions.lvarstransfer.transfer);

                                    if let Err(e) = events::control::GainControlEvent.emit(&app_handle){
                                        log::error!("Could not emit GainControlEvent: {:?}", e);
                                    }

                                    // Not really used by the host
                                    connection_time = Some(Instant::now());
                                } else {
                                    // Display connected message

                                    // TODO
                                    // app_interface.connected();
                                    if let Err(e) = events::control::LoseControlEvent.emit(&app_handle){
                                        log::error!("Could not emit LoseControlEvent: {:?}", e);
                                    }
                                }
                            }
                            Event::ConnectionLost(reason) => {
                                log::info!("Server/Client stopped. Reason: {}", reason);
                                // TAKEING BACK CONTROL
                                control.take_control(conn, &definitions.lvarstransfer.transfer);

                                clients.reset();
                                observing = false;
                                should_set_none_client = true;

                                // TODO
                                // if let Err(e) = audio.play_disconnected() {
                                //     log::warn!("[AUDIO] Error playing audio: {}", e);
                                // }

                                if let Err(e) = events::ClientFailEvent(reason).emit(&app_handle){
                                    log::error!("Could not emit ClientFailEvent: {:?}", e);
                                }
                            }
                            Event::UnablePunchthrough => {
                                if let Err(e) = events::ClientFailEvent("Could not connect to host! Please port forward or use 'Cloud Host'!".to_string()).emit(&app_handle){
                                    log::error!("Could not emit ClientFailEvent: {:?}", e);
                                }
                            }
                            Event::SessionIdFetchFailed => {
                                if let Err(e) = events::server::ServerFailEvent("Could not connect to Cloud Server to fetch session ID.".to_string()).emit(&app_handle){
                                    log::error!("Could not emit ServerFailEvent: {:?}", e);
                                }
                            }
                            Event::Metrics(metrics) => {
                                if let Err(e) = events::MetricsEvent::from(metrics).emit(&app_handle){
                                    log::error!("Could not emit MetricsEvent: {:?}", e);
                                }
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
                        definitions::ProgramAction::TakeControls => {
                            if !control.has_control() && !observing {
                                if let Some(in_control) = clients.get_client_in_control() {
                                    control.take_control(conn, &definitions.lvarstransfer.transfer);
                                    client.take_control(in_control.clone());
                                }
                            }
                        }
                        definitions::ProgramAction::TransferControls => {
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
                        let permission = definitions::SyncPermission {
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
        }
    });

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub fn connect(
    app_handle: tauri::AppHandle,
    sim_connect_state: State<'_, states::SimConnectorState>,
    definitions_state: State<'_, states::DefinitionsState>,
    transfer_client_state: State<'_, states::TransferClientState>,
    username: String,
    session_id: Option<String>,
    isipv6: bool,
    ip: Option<net::IpAddr>,
    hostname: Option<String>,
    port: Option<u16>,
    method: ConnectionMethod,
) -> Result<()> {
    // TODO
    let updater = Updater::new();
    let config = Config::default();

    {
        let mut sim_connect_mutex = sim_connect_state.lock().unwrap();
        let mut definitions_mutex = definitions_state.lock().unwrap();

        connect_to_sim(&mut sim_connect_mutex.0, &mut definitions_mutex.0)?;
    }

    // Display attempting to start server
    // TODO
    // app_interface.attempt();

    match start_client(
        config.conn_timeout,
        username.clone(),
        session_id,
        updater.get_version().to_string(),
        isipv6,
        ip,
        hostname,
        port,
        method,
    ) {
        Ok(new_client) => {
            log::info!("Client started.");

            *transfer_client_state.lock().unwrap() =
                Some(states::TransferClientWrapper(Box::new(new_client)));
        }
        Err(e) => {
            // TODO: maybe don't use events here

            if let Err(e) = events::client_fail::ClientFailEvent(e.to_string()).emit(&app_handle) {
                log::error!("Could not emit ClientFailEvent: {:?}", e);
            }

            log::error!("Could not start client! Reason: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn disconnect(transfer_client_state: State<'_, states::TransferClientState>) {
    log::info!("Request to disconnect.");

    let mut transfer_client_mutex = transfer_client_state.lock().unwrap();
    if let Some(client) = transfer_client_mutex.as_mut() {
        client.0.stop("Stopped.".to_string());
    }

    *transfer_client_mutex = None;
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
pub fn set_observer(
    client_manager_state: State<'_, states::ClientManagerState>,
    transfer_client: State<'_, states::TransferClientState>,
    target: String,
    is_observer: bool,
) {
    client_manager_state
        .lock()
        .unwrap()
        .set_observer(&target, is_observer);

    if let Some(client) = transfer_client.lock().unwrap().as_ref() {
        log::info!("Setting {} as observer. {}", target, is_observer);
        client.0.set_observer(target, is_observer);
    }
}

#[tauri::command]
#[specta::specta]
pub fn go_observer(transfer_client_state: State<'_, states::TransferClientState>) {
    if let Some(client) = transfer_client_state.lock().unwrap().as_ref() {
        log::info!("Request to set self as observer.");
        client.0.set_self_observer();
    }
}

#[tauri::command]
#[specta::specta]
pub fn force_take_control(
    transfer_client_state: State<'_, states::TransferClientState>,
    client_manager_state: State<'_, states::ClientManagerState>,
) {
    if let Some(client) = transfer_client_state.lock().unwrap().as_ref() {
        if let Some(client_name) = client_manager_state.lock().unwrap().get_client_in_control() {
            //Will send a loopback Payloads::TransferControl
            client.0.take_control(client_name.clone())
        }
    }
}

fn load_aircraft() {}
// AppMessage::LoadAircraft { config_file_name } => {
//     // Load config
//     info!(
//         "{} aircraft config selected.",
//         config_file_name
//     );
//     config_to_load.clone_from(&config_file_name);
// }

fn startup() {}
// AppMessage::Startup => {
//     // List aircraft
//     if let Ok(configs) = get_aircraft_configs() {
//         info!(
//             "Found {} configuration file(s).",
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

#[tauri::command]
#[specta::specta]
pub async fn get_public_ip() -> Result<net::IpAddr> {
    let local_ip = local_ip_address::local_ip()?;

    let search_options = igd_next::SearchOptions {
        bind_addr: (local_ip, 0).into(),
        ..Default::default()
    };

    let ip = igd_next::aio::tokio::search_gateway(search_options)
        .await?
        .get_external_ip()
        .await?;

    Ok(ip)
}
