#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod app;
mod audio;
mod clientmanager;
mod corrector;
mod definitions;
mod simconfig;
mod sync;
mod syncdefs;
mod update;
mod util;
mod varreader;

use app::{App, AppMessage, ConnectionMethod};
use audio::AudioManager;
use clientmanager::ClientManager;
use definitions::{Definitions, ProgramAction, SyncPermission};
use log::{error, info, warn};
use simconfig::Config;
use simconnect::{DispatchResult, SimConnector};

use spin_sleep::sleep;
use std::{
    env,
    fs::{read_dir, File},
    io,
    net::IpAddr,
    path::PathBuf,
    time::Duration,
    time::Instant,
};
use update::Updater;
use yourcontrols_net::{Client, Event, Payloads, ReceiveMessage, Server, TransferClient};
use yourcontrols_types::AllNeedSync;

use crate::util::get_hostname_ip;

use control::*;
use sync::*;

const LOG_FILENAME: &str = "log.txt";
const CONFIG_FILENAME: &str = "config.json";
const AIRCRAFT_DEFINITIONS_PATH: &str = "definitions/aircraft/";

const LOOP_SLEEP_TIME: Duration = Duration::from_millis(10);

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
        Ok(_) => {}
        Err(e) => error!(
            "[PROGRAM] Could not write configuration file! Reason: {}",
            e
        ),
    };
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
) -> Result<Client, String> {
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
        ConnectionMethod::Relay => panic!("Never should be reached!"),
    };

    match client_result {
        Ok(_) => Ok(client),
        Err(e) => Err(format!("Could not start client! Reason: {}", e)),
    }
}

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
            info!("[PACKET] SENT {:?}", data);
        }

        client.update(data, false);
    }
}

fn main() {
    let is_dev_build = cfg!(debug_assertions);

    if !is_dev_build {
        // Set CWD to application directory
        let exe_path = env::current_exe();
        env::set_current_dir(exe_path.unwrap().parent().unwrap()).ok();
    }
    // Initialize logging
    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        File::create(LOG_FILENAME).unwrap(),
    )
    .ok();
    // Load configuration file
    let mut config = match Config::read_from_file(CONFIG_FILENAME) {
        Ok(config) => config,
        Err(e) => {
            warn!(
                "[PROGRAM] Could not open config. Using default values. Reason: {}",
                e
            );

            let config = Config::default();
            write_configuration(&config);
            config
        }
    };

    let mut conn = simconnect::SimConnector::new();
    let mut control = Control::new();
    let mut clients = ClientManager::new();

    let mut audio = AudioManager::new();
    if let Err(e) = audio.setup_stream() {
        error!("[AUDIO] Could not initialize audio! Reason: {}", e);
    }
    audio.mute(config.sound_muted);

    let mut updater = Updater::new();
    let mut installer_spawned = false;

    // Set up sim connect
    let mut observing = false;
    // Client stopped, need to stop transfer client
    let mut should_set_none_client = false;

    let app_interface = App::setup(format!("YourControls v{}", updater.get_version()));

    // Transfer
    let mut transfer_client: Option<Box<dyn TransferClient>> = None;

    // Update rate counter
    let mut definitions = Definitions::new();

    let mut ready_to_process_data = false;

    let mut connection_time = None;

    let mut config_to_load = String::new();
    // Helper closures
    let get_config_path = |config_name: &str| -> PathBuf {
        let mut path = PathBuf::from(AIRCRAFT_DEFINITIONS_PATH);
        path.push(config_name);
        path
    };
    // Load defintions
    let load_definitions = |definitions: &mut Definitions, config_to_load: &mut String| -> bool {
        // Load aircraft configuration
        let path = get_config_path(config_to_load);

        match definitions.load_config(path.to_string_lossy().to_string()) {
            Ok(_) => {
                info!("[DEFINITIONS] Loaded and mapped {} aircraft vars, {} local vars, and {} events", definitions.get_number_avars(), definitions.get_number_lvars(), definitions.get_number_events());
            }
            Err(e) => {
                error!(
                    "[DEFINITIONS] Could not load configuration file {}: {}",
                    config_to_load, e
                );
                // Prevent server/client from starting as config could not be laoded.
                *config_to_load = String::new();
                return false;
            }
        };

        info!("[DEFINITIONS] {} loaded successfully.", config_to_load);

        true
    };

    let connect_to_sim = |conn: &mut SimConnector, definitions: &mut Definitions| {
        // Connect to simconnect
        *definitions = Definitions::new();
        #[cfg(not(feature = "skip_sim_connect"))]
        let connected = conn.connect("YourControls");
        #[cfg(feature = "skip_sim_connect")]
        let connected = true;
        if connected {
            // Display not connected to server message
            info!("[SIM] Connected to SimConnect.");
        } else {
            // Display trying to connect message
            app_interface.error("Could not connect to SimConnect! Is the sim running?");
        };

        connected
    };

    loop {
        let timer = Instant::now();

        if let Some(client) = transfer_client.as_mut() {
            // Simconnect message
            while let Ok(message) = conn.get_next_message() {
                match message {
                    DispatchResult::SimobjectData(data) => {
                        definitions.process_sim_object_data(data);
                    }
                    // Exception occured
                    DispatchResult::Exception(data) => {
                        warn!("[SIM] SimConnect exception occurred: {}", unsafe {
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
                    _ => {}
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
                                info!(
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
                                    &conn,
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
                                info!("[CONTROL] Taking control from {}", from);
                                control.take_control(&conn, &definitions.lvarstransfer.transfer);
                                app_interface.gain_control();
                                clients.set_no_control();
                            // Someone else has controls, if we have controls we let go and listen for their messages
                            } else {
                                if from == client.get_server_name() {
                                    app_interface.lose_control();
                                    control
                                        .lose_control(&conn, &definitions.lvarstransfer.transfer);
                                }
                                info!("[CONTROL] {} is now in control.", to);
                                app_interface.set_incontrol(&to);
                                clients.set_client_control(to);
                            }
                        }
                        Payloads::PlayerJoined {
                            name,
                            in_control,
                            mut is_observer,
                            is_server,
                        } => {
                            info!(
                                "[NETWORK] {} connected. In control: {}, observing: {}, server: {}",
                                name, in_control, is_observer, is_server
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

                            app_interface.new_connection(&name);
                            app_interface.set_observing(&name, is_observer);
                            clients.set_server(&name, is_server);
                            clients.set_observer(&name, is_observer);

                            if in_control {
                                app_interface.set_incontrol(&name);
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
                            info!("[NETWORK] {} lost connection.", name);

                            clients.remove_client(&name);
                            // User may have been in control
                            if clients.client_has_control(&name) {
                                clients.set_no_control();
                                // Transfer control to myself if I'm server
                                if client.is_host() {
                                    info!("[CONTROL] {} had control, taking control back.", name);
                                    app_interface.gain_control();

                                    control
                                        .take_control(&conn, &definitions.lvarstransfer.transfer);
                                    client.transfer_control(client.get_server_name().to_string());
                                }
                            }

                            app_interface.lost_connection(&name);
                        }
                        Payloads::SetObserver {
                            from: _,
                            to,
                            is_observer,
                        } => {
                            if to == client.get_server_name() {
                                info!("[CONTROL] Server set us to observing? {}", is_observer);
                                observing = is_observer;
                                app_interface.observing(is_observer);

                                if !observing {
                                    definitions.reset_sync();
                                }
                            } else {
                                info!("[CONTROL] {} is observing? {}", to, is_observer);
                                clients.set_observer(&to, is_observer);
                                app_interface.set_observing(&to, is_observer);
                            }
                        }
                        Payloads::SetHost => {
                            app_interface.set_host();
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
                                    info!("[DEFINITIONS] Loaded and mapped {} aircraft vars, {} local vars, and {} events from the server", definitions.get_number_avars(), definitions.get_number_lvars(), definitions.get_number_events());
                                    control.on_connected(&conn);

                                    let def_connect_result = definitions.on_connected(&conn);
                                    if let Err(()) = def_connect_result {
                                        client.stop("Error starting WS server. Do you have another YourControls open?".to_string())
                                    }
                                    // Freeze aircraft
                                    control
                                        .lose_control(&conn, &definitions.lvarstransfer.transfer);
                                }
                                Err(e) => {
                                    error!("[DEFINITIONS] Could not load server sent configuration file: {}", e);
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
                                    info!(
                                        "[NETWORK] New client started to connect to hosted server."
                                    );
                                    *client = Box::new(new_client);
                                }
                                Err(e) => {
                                    app_interface.client_fail(e.to_string().as_str());
                                    error!(
                                        "[NETWORK] Could not start new hoster client! Reason: {}",
                                        e
                                    );
                                }
                            };
                        }
                        Payloads::SetSelfObserver { name } => {
                            if client.is_host() {
                                clients.set_observer(&name, true);
                                app_interface.set_observing(&name, true);
                                client.set_observer(name, true);
                            }
                        }
                    },
                    ReceiveMessage::Event(e) => match e {
                        Event::ConnectionEstablished => {
                            if client.is_host() {
                                // Display server started message
                                app_interface.server_started();
                                if let Some(session_code) = client.get_session_id().as_deref() {
                                    app_interface.set_session_code(session_code);
                                }
                                // Unfreeze aircraft
                                control.take_control(&conn, &definitions.lvarstransfer.transfer);
                                app_interface.gain_control();
                                // Not really used by the host
                                connection_time = Some(Instant::now());
                            } else {
                                // Display connected message
                                app_interface.connected();
                                app_interface.lose_control();
                            }
                        }
                        Event::ConnectionLost(reason) => {
                            info!("[NETWORK] Server/Client stopped. Reason: {}", reason);
                            // TAKE BACK CONTROL
                            control.take_control(&conn, &definitions.lvarstransfer.transfer);

                            clients.reset();
                            observing = false;
                            should_set_none_client = true;

                            if let Err(e) = audio.play_disconnected() {
                                warn!("[AUDIO] Error playing audio: {}", e);
                            }

                            app_interface.client_fail(&reason);
                        }
                        Event::UnablePunchthrough => app_interface.client_fail(
                            "Could not connect to host! Please port forward or use 'Cloud Host'!",
                        ),

                        Event::SessionIdFetchFailed => app_interface
                            .server_fail("Could not connect to Cloud Server to fetch session ID."),

                        Event::Metrics(metrics) => {
                            app_interface.send_network(&metrics);
                        }
                    },
                }
            }

            if let Err(e) = definitions.step(&conn) {
                client.stop(e.to_string());
            }

            // Handle specific program triggered actions
            if let Some(pending_action) = definitions.get_next_pending_action() {
                match pending_action {
                    ProgramAction::TakeControls => {
                        if !control.has_control() && !observing {
                            if let Some(in_control) = clients.get_client_in_control() {
                                control.take_control(&conn, &definitions.lvarstransfer.transfer);
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
                            control.take_control(&conn, &definitions.lvarstransfer.transfer);
                            client.take_control(in_control.clone());
                        }
                    }
                }
            }

            // Handle initial 3 second connection delay, allows lvars to be processed
            if let Some(true) = connection_time.map(|t| t.elapsed().as_secs() >= 3) {
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
        }

        // GUI
        if let Ok(msg) = app_interface.get_next_message() {
            match msg {
                AppMessage::StartServer {
                    username,
                    port,
                    is_ipv6,
                    method,
                    use_upnp,
                } => {
                    let connected = connect_to_sim(&mut conn, &mut definitions);

                    if config_to_load.is_empty() {
                        app_interface.server_fail("Select an aircraft config first!");
                    } else if !load_definitions(&mut definitions, &mut config_to_load) {
                        app_interface.error(
                            "Error loading definition files. Check the log for more information.",
                        );
                    } else if connected {
                        definitions.on_connected(&conn).ok();
                        control.on_connected(&conn);
                        // Display attempting to start server
                        app_interface.attempt();

                        match method {
                            ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
                                let mut server = Box::new(Server::new(
                                    username.clone(),
                                    updater.get_version().to_string(),
                                    config.conn_timeout,
                                ));

                                let result = match method {
                                    ConnectionMethod::Direct => {
                                        server.start(is_ipv6, port, use_upnp)
                                    }
                                    ConnectionMethod::CloudServer => {
                                        server.start_with_hole_punching(is_ipv6)
                                    }
                                    _ => panic!("Not implemented!"),
                                };

                                match result {
                                    Ok(_) => {
                                        // Assign server as transfer client
                                        transfer_client = Some(server);
                                        info!("[NETWORK] Server started");
                                    }
                                    Err(e) => {
                                        app_interface.server_fail(&e.to_string());
                                        info!("[NETWORK] Could not start server! Reason: {}", e);
                                    }
                                }
                            }
                            ConnectionMethod::Relay => {
                                let mut client = Box::new(Client::new(
                                    username.clone(),
                                    updater.get_version().to_string(),
                                    config.conn_timeout,
                                ));

                                match client.start_with_relay(is_ipv6) {
                                    Ok(_) => {
                                        transfer_client = Some(client);
                                        info!("[NETWORK] Hosting started");
                                    }
                                    Err(e) => {
                                        info!("[NETWORK] Hosting could not start! Reason: {}", e);
                                        app_interface.server_fail(&e.to_string());
                                    }
                                }
                            }
                        };

                        config.port = port;
                        config.name = username;
                        write_configuration(&config);
                    }
                }
                AppMessage::Connect {
                    session_id,
                    username,
                    method,
                    ip,
                    port,
                    isipv6,
                    hostname,
                } => {
                    let connected = connect_to_sim(&mut conn, &mut definitions);

                    if connected {
                        // Display attempting to start server
                        app_interface.attempt();

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
                            Ok(client) => {
                                info!("[NETWORK] Client started.");
                                transfer_client = Some(Box::new(client));
                            }
                            Err(e) => {
                                app_interface.client_fail(e.to_string().as_str());
                                error!("[NETWORK] Could not start client! Reason: {}", e);
                            }
                        }

                        // Write config with new values
                        config.name = username;
                        config.port = port.unwrap_or(config.port);
                        config.ip = if let Some(ip) = ip {
                            ip.to_string()
                        } else {
                            String::new()
                        };
                        write_configuration(&config);
                    }
                }
                AppMessage::Disconnect => {
                    info!("[NETWORK] Request to disconnect.");
                    if let Some(client) = transfer_client.as_mut() {
                        client.stop("Stopped.".to_string());
                    }
                }
                AppMessage::TransferControl { target } => {
                    if let Some(client) = transfer_client.as_ref() {
                        info!("[CONTROL] Giving control to {}", target);
                        // Send server message, will send a loopback Payloads::TransferControl
                        client.transfer_control(target.clone());
                    }
                }
                AppMessage::SetObserver {
                    target,
                    is_observer,
                } => {
                    clients.set_observer(&target, is_observer);
                    if let Some(client) = transfer_client.as_ref() {
                        info!("[CONTROL] Setting {} as observer. {}", target, is_observer);
                        client.set_observer(target, is_observer);
                    }
                }
                AppMessage::GoObserver => {
                    if let Some(client) = transfer_client.as_ref() {
                        if let Some(client_name) = clients.get_client_in_control() {
                            // Requests server to set self as observer
                            client.set_self_observer();
                        }
                    }
                }
                AppMessage::LoadAircraft { config_file_name } => {
                    // Load config
                    info!(
                        "[DEFINITIONS] {} aircraft config selected.",
                        config_file_name
                    );
                    config_to_load = config_file_name.clone();
                }
                AppMessage::Startup => {
                    // List aircraft
                    if let Ok(configs) = get_aircraft_configs() {
                        info!(
                            "[DEFINITIONS] Found {} configuration file(s).",
                            configs.len()
                        );

                        for aircraft_config in configs {
                            app_interface.add_aircraft(&aircraft_config);
                        }
                    }

                    app_interface.send_config(&config.get_json_string());
                    // Update version
                    let app_version = updater.get_version();
                    if let Ok(newest_version) = updater.get_latest_version() {
                        if *newest_version > app_version
                            && (newest_version.pre.is_empty()
                                || newest_version.pre.is_empty() && config.check_for_betas)
                        {
                            app_interface.version(&newest_version.to_string());
                        }
                        info!(
                            "[UPDATER] Version {} in use, {} is newest.",
                            app_version, newest_version
                        )
                    } else {
                        info!("[UPDATER] Version {} in use.", app_version)
                    }
                }
                AppMessage::RunUpdater => {
                    match updater.run_installer() {
                        Ok(_) => {
                            // Terminate self
                            installer_spawned = true
                        }
                        Err(e) => {
                            error!("[UPDATER] Downloading installer failed. Reason: {}", e);
                            app_interface.update_failed();
                        }
                    };
                }
                AppMessage::UpdateConfig { new_config: config } => {
                    audio.mute(config.sound_muted);
                    write_configuration(&config);
                }
                AppMessage::ForceTakeControl => {
                    if let Some(client) = transfer_client.as_ref() {
                        if let Some(client_name) = clients.get_client_in_control() {
                            //Will send a loopback Payloads::TransferControl
                            client.take_control(client_name.clone())
                        }
                    }
                }
            }
        }

        if should_set_none_client {
            // Prevent sending any more data
            transfer_client = None;
            should_set_none_client = false;
            ready_to_process_data = false;
            connection_time = None;
            conn.close();
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
