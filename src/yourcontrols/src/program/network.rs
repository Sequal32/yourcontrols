use std::net::IpAddr;
use std::time::Instant;

use log::{error, info};
use yourcontrols_net::{Client, Event, Payloads, ReceiveMessage, TransferClient};

use crate::app::App;
use crate::app::ConnectionMethod;
use crate::cli::CliWrapper;
use crate::clientmanager::ClientManager;
use crate::definitions::SyncPermission;
use crate::simconfig::Config;
use crate::update::Updater;
use crate::util::get_hostname_ip;

use super::emulator_runtime::EmulatorController;
use super::emulator_runtime::EmulatorRuntimeState;
use super::simconnect::{SimAction, SimState};
use super::sync::SyncState;

pub struct NetworkState {
    pub(crate) clients: ClientManager,
    pub(crate) transfer_client: Option<Box<dyn TransferClient>>,
    pub(crate) observing: bool,
    pub(crate) should_set_none_client: bool,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            clients: ClientManager::new(),
            transfer_client: None,
            observing: false,
            should_set_none_client: false,
        }
    }

    pub fn has_client(&self) -> bool {
        self.transfer_client.is_some()
    }
}

pub struct NetworkContext<'a> {
    pub emulator: &'a EmulatorRuntimeState,
    pub sim: &'a mut SimState,
    pub config: &'a Config,
    pub cli: &'a CliWrapper,
    pub updater: &'a Updater,
    pub app: &'a mut App,
    pub sync: &'a mut SyncState,
}

pub struct NetworkController;

impl NetworkController {
    pub fn poll(state: &mut NetworkState, ctx: &mut NetworkContext<'_>) {
        let Some(mut client) = state.transfer_client.take() else {
            return;
        };

        while let Ok(message) = client.get_next_message() {
            NetworkHandler::handle_message(&mut client, message, state, ctx);
        }

        state.transfer_client = Some(client);
    }

    pub fn apply_sim_action(state: &mut NetworkState, action: SimAction) {
        let SimAction::StopTransfer(reason) = action;
        if let Some(client) = state.transfer_client.as_mut() {
            client.stop(reason);
        }
    }

    pub fn cleanup_if_needed(state: &mut NetworkState, sync: &mut SyncState, sim: &mut SimState) {
        if !state.should_set_none_client {
            return;
        }

        state.transfer_client = None;
        state.should_set_none_client = false;
        sync.ready_to_process_data = false;
        sync.connection_time = None;
        sim.conn.close();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_client(
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
            ConnectionMethod::CloudServer => {
                client.start_with_hole_punch(session_id.unwrap(), isipv6)
            }
            ConnectionMethod::Relay => panic!("Never should be reached!"),
        };

        match client_result {
            Ok(_) => Ok(client),
            Err(e) => Err(format!("Could not start client! Reason: {}", e)),
        }
    }
}

pub struct NetworkHandler;

impl NetworkHandler {
    fn handle_message(
        client: &mut Box<dyn TransferClient>,
        message: ReceiveMessage,
        state: &mut NetworkState,
        ctx: &mut NetworkContext<'_>,
    ) {
        match message {
            ReceiveMessage::Payload(payload) => {
                Self::handle_payload(client, payload, state, ctx);
            }
            ReceiveMessage::Event(event) => {
                Self::handle_event(client, event, state, ctx);
            }
        }
    }

    fn handle_payload(
        client: &mut Box<dyn TransferClient>,
        payload: Payloads,
        state: &mut NetworkState,
        ctx: &mut NetworkContext<'_>,
    ) {
        match payload {
            // Unused
            Payloads::Handshake { .. }
            | Payloads::RendezvousHandshake { .. }
            | Payloads::HostingReceived { .. }
            | Payloads::AttemptConnection { .. }
            | Payloads::PeerEstablished { .. }
            | Payloads::InvalidVersion { .. }
            | Payloads::InvalidName
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
                        state.clients.is_observer(&from),
                        state.clients.client_is_server(&from),
                        state.clients.client_has_control(&from)
                    );
                }

                if !state.clients.is_observer(&from) && ctx.sync.ready_to_process_data {
                    match ctx.sim.definitions.on_receive_data(
                        &ctx.sim.conn,
                        data,
                        time,
                        &SyncPermission {
                            is_server: state.clients.client_is_server(&from),
                            is_master: state.clients.client_has_control(&from),
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
                ctx.sim.definitions.reset_sync();
                if to == client.get_server_name() {
                    info!("[CONTROL] Taking control from {}", from);
                    ctx.sim.take_control();
                    ctx.app.gain_control();
                    state.clients.set_no_control();
                // Someone else has controls, if we have controls we let go and listen for their messages
                } else {
                    if from == client.get_server_name() {
                        ctx.app.lose_control();
                        ctx.sim.lose_control();
                    }
                    info!("[CONTROL] {} is now in control.", to);
                    ctx.app.set_incontrol(&to);
                    state.clients.set_client_control(to);
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
                state.clients.add_client(name.clone());

                if client.is_host() {
                    client.send_definitions(
                        ctx.sim.definitions.get_buffer_bytes().into_boxed_slice(),
                        name.clone(),
                    );

                    if ctx.config.instructor_mode {
                        is_observer = true;
                        client.set_observer(name.clone(), true);
                    }
                }

                ctx.app.new_connection(&name);
                ctx.app.set_observing(&name, is_observer);
                state.clients.set_server(&name, is_server);
                state.clients.set_observer(&name, is_observer);

                if in_control {
                    ctx.app.set_incontrol(&name);
                    state.clients.set_client_control(name);
                }
            }
            // Person is ready to receive data
            Payloads::Ready => {
                if ctx.sim.definitions.has_control() {
                    client.update(ctx.sim.definitions.get_all_current(), false);
                }
                // Request time update to sync
                if client.is_host() {
                    ctx.sim.definitions.request_time();
                }
            }
            Payloads::PlayerLeft { name } => {
                info!("[NETWORK] {} lost connection.", name);

                state.clients.remove_client(&name);
                // User may have been in control
                if state.clients.client_has_control(&name) {
                    state.clients.set_no_control();
                    // Transfer control to myself if I'm server
                    if client.is_host() {
                        info!("[CONTROL] {} had control, taking control back.", name);
                        ctx.app.gain_control();
                        ctx.sim.take_control();
                        client.transfer_control(client.get_server_name().to_string());
                    }
                }

                ctx.app.lost_connection(&name);
            }
            Payloads::SetObserver {
                from: _,
                to,
                is_observer,
            } => {
                if to == client.get_server_name() {
                    info!("[CONTROL] Server set us to observing? {}", is_observer);
                    state.observing = is_observer;
                    ctx.app.observing(is_observer);

                    if !state.observing {
                        ctx.sim.definitions.reset_sync();
                    }
                } else {
                    info!("[CONTROL] {} is observing? {}", to, is_observer);
                    state.clients.set_observer(&to, is_observer);
                    ctx.app.set_observing(&to, is_observer);
                }
            }
            Payloads::SetHost => {
                ctx.app.set_host();
                // Host was set which means successfully established connection to hoster, need to send definitions
                client.send_definitions(
                    ctx.sim.definitions.get_buffer_bytes().into_boxed_slice(),
                    client.get_server_name().to_string(),
                );
            }
            Payloads::ConnectionDenied { reason } => {
                client.stop(format!("Connection denied: {}", reason));
            }
            Payloads::AircraftDefinition { bytes } => {
                match ctx.sim.definitions.load_config_from_bytes(bytes) {
                    Ok(_) => {
                        info!("[DEFINITIONS] Loaded and mapped {} aircraft vars, {} local vars, and {} events from the server.",
                        ctx.sim.definitions.get_number_avars(), ctx.sim.definitions.get_number_lvars(), ctx.sim.definitions.get_number_events());

                        let skip_sim_connect = ctx.cli.skip_sim_connect();
                        let def_connect_result = ctx
                            .sim
                            .definitions
                            .on_connected(&ctx.sim.conn, skip_sim_connect);
                        if let Err(()) = def_connect_result {
                            client.stop(
                                "Error starting WS server. Do you have another YourControls open?"
                                    .to_string(),
                            )
                        }
                        // Freeze aircraft
                        ctx.sim.lose_control();
                    }
                    Err(e) => {
                        error!(
                            "[DEFINITIONS] Could not load server sent configuration file: {}",
                            e
                        );
                    }
                }

                EmulatorController::send_vars_if_enabled(
                    ctx.emulator,
                    &ctx.sim.definitions,
                    ctx.app,
                );
                // Start the connection timer to wait to send the ready payload
                ctx.sync.connection_time = Some(Instant::now());
            }
            Payloads::AttemptHosterConnection { peer } => {
                match NetworkController::start_client(
                    ctx.config.conn_timeout,
                    client.get_server_name().to_string(),
                    client.get_session_id(),
                    ctx.updater.get_version().to_string(),
                    false,
                    Some(peer.ip()),
                    None,
                    Some(peer.port()),
                    ConnectionMethod::Direct,
                ) {
                    Ok(new_client) => {
                        info!("[NETWORK] New client started to connect to hosted server.");
                        *client = Box::new(new_client);
                    }
                    Err(e) => {
                        ctx.app.client_fail(e.to_string().as_str());
                        error!("[NETWORK] Could not start new hoster client! Reason: {}", e);
                    }
                };
            }
            Payloads::SetSelfObserver { name } => {
                if client.is_host() {
                    state.clients.set_observer(&name, true);
                    ctx.app.set_observing(&name, true);
                    client.set_observer(name, true);
                }
            }
        }
    }

    fn handle_event(
        client: &mut Box<dyn TransferClient>,
        event: Event,
        state: &mut NetworkState,
        ctx: &mut NetworkContext<'_>,
    ) {
        match event {
            Event::ConnectionEstablished => {
                if client.is_host() {
                    // Display server started message
                    ctx.app.server_started();
                    if let Some(session_code) = client.get_session_id().as_deref() {
                        ctx.app.set_session_code(session_code);
                    }
                    // Unfreeze aircraft
                    ctx.sim.take_control();
                    ctx.app.gain_control();
                    // Not really used by the host
                    ctx.sync.connection_time = Some(Instant::now());
                } else {
                    // Display connected message
                    ctx.app.connected();
                    ctx.app.lose_control();
                }
            }
            Event::ConnectionLost(reason) => {
                info!("[NETWORK] Server/Client stopped. Reason: {}", reason);
                // TAKE BACK CONTROL
                ctx.sim.take_control();

                state.clients.reset();
                state.observing = false;
                state.should_set_none_client = true;

                ctx.app.client_fail(&reason);
            }
            Event::UnablePunchthrough => ctx
                .app
                .client_fail("Could not connect to host! Please port forward or use Cloud Host."),
            Event::SessionIdFetchFailed => ctx
                .app
                .server_fail("Could not connect to Cloud Server to fetch session ID."),
            Event::Metrics(metrics) => {
                ctx.app.send_network(&metrics);
            }
        }
    }
}
