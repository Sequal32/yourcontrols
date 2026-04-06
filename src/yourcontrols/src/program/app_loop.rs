use log::{error, info};
use yourcontrols_net::{Client, Server};

use crate::app::{App, AppMessage, ConnectionMethod};
use crate::cli::CliWrapper;
use crate::definitions::Definitions;
use crate::paths::DefinitionPathResolver;
use crate::simconfig::Config;
use crate::update::Updater;

use super::emulator_runtime::{EmulatorController, EmulatorSetContext};
use super::network::{NetworkController, NetworkState};
use super::simconnect::SimState;
use super::state::ProgramState;
use super::StartServerParameters;

pub struct AppState {
    pub(crate) app_interface: App,
    pub(crate) installer_spawned: bool,
    pub(crate) definitions_to_load: String,
}

impl AppState {
    pub fn new(app_interface: App) -> Self {
        Self {
            app_interface,
            installer_spawned: false,
            definitions_to_load: String::new(),
        }
    }
}

pub struct AppContext<'a> {
    pub program_state: &'a mut ProgramState,
    pub sim: &'a mut SimState,
    pub network: &'a mut NetworkState,
    pub config: &'a mut Config,
    pub cli: &'a CliWrapper,
    pub updater: &'a mut Updater,
}

pub struct AppController;

impl AppController {
    pub fn poll(state: &mut AppState, ctx: &mut AppContext<'_>) {
        if let Ok(msg) = state.app_interface.get_next_message() {
            AppHandler::handle_message(msg, state, ctx);
        }
    }
}

pub struct AppHandler;

impl AppHandler {
    fn handle_message(msg: AppMessage, state: &mut AppState, ctx: &mut AppContext<'_>) {
        match msg {
            AppMessage::StartServer {
                username,
                port,
                is_ipv6,
                method,
                use_upnp, // Defaults to true in the UI
            } => {
                let server_params = StartServerParameters {
                    method,
                    is_ipv6,
                    use_upnp,
                };
                ctx.config.name = username.clone();
                ctx.config.port = port;

                Self::handle_start_server(state, ctx, &server_params);
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
                let connected = Self::connect_to_sim(state, ctx);

                if connected {
                    // Display attempting to start server
                    state.app_interface.attempt();

                    match NetworkController::start_client(
                        ctx.config.conn_timeout,
                        username.clone(),
                        session_id,
                        ctx.updater.get_version().to_string(),
                        isipv6,
                        ip,
                        hostname,
                        port,
                        method,
                    ) {
                        Ok(client) => {
                            info!("[NETWORK] Client started.");
                            ctx.network.transfer_client = Some(Box::new(client));
                        }
                        Err(e) => {
                            state.app_interface.client_fail(e.to_string().as_str());
                            error!("[NETWORK] Could not start client! Reason: {}", e);
                        }
                    }

                    // Write config with new values
                    ctx.config.name = username;
                    ctx.config.port = port.unwrap_or(ctx.config.port);
                    ctx.config.ip = if let Some(ip) = ip {
                        ip.to_string()
                    } else {
                        String::new()
                    };

                    ctx.config.write();
                }
            }
            AppMessage::Disconnect => {
                info!("[NETWORK] Request to disconnect.");
                if let Some(client) = ctx.network.transfer_client.as_mut() {
                    client.stop("Stopped.".to_string());
                }
            }
            AppMessage::TransferControl { target } => {
                if let Some(client) = ctx.network.transfer_client.as_ref() {
                    info!("[CONTROL] Giving control to {}", target);
                    // Send server message, will send a loopback Payloads::TransferControl
                    client.transfer_control(target.clone());
                }
            }
            AppMessage::SetObserver {
                target,
                is_observer,
            } => {
                ctx.network.clients.set_observer(&target, is_observer);
                if let Some(client) = ctx.network.transfer_client.as_ref() {
                    info!("[CONTROL] Setting {} as observer. {}", target, is_observer);
                    client.set_observer(target, is_observer);
                }
            }
            AppMessage::GoObserver => {
                if let Some(client) = ctx.network.transfer_client.as_ref() {
                    // Requests server to set self as observer
                    client.set_self_observer();
                }
            }
            AppMessage::LoadAircraft {
                config_file_name,
                sim,
            } => {
                // Load config
                info!(
                    "[DEFINITIONS] {} aircraft config selected for {}.",
                    config_file_name, sim
                );
                if let Some(definition_path) =
                    DefinitionPathResolver::from_sim_and_config(&sim, &config_file_name)
                {
                    info!(
                        "[DEFINITIONS] Will load definition file {} for {}.",
                        definition_path.display(),
                        sim
                    );
                    state.definitions_to_load = definition_path.to_string_lossy().to_string();
                } else {
                    error!(
                        "[DEFINITIONS] Could not find definition file for {} config {}!",
                        sim, config_file_name
                    );
                }
            }
            AppMessage::Startup => {
                // List aircraft
                if let Ok(configs) = DefinitionPathResolver::get_fs_2020_configs() {
                    info!(
                        "[DEFINITIONS] Found {} FS2020 configuration file(s).",
                        configs.len()
                    );

                    for aircraft_config in configs {
                        state.app_interface.add_fs2020_aircraft(&aircraft_config);
                    }
                }
                if let Ok(configs) = DefinitionPathResolver::get_fs_2024_configs() {
                    info!(
                        "[DEFINITIONS] Found {} FS2024 configuration file(s).",
                        configs.len()
                    );

                    for aircraft_config in configs {
                        state.app_interface.add_fs2024_aircraft(&aircraft_config);
                    }
                }

                state
                    .app_interface
                    .send_config(&ctx.config.get_json_string());
                // Update version
                let app_version = ctx.updater.get_version();
                if let Ok(newest_version) = ctx.updater.get_latest_version() {
                    if *newest_version > app_version && newest_version.pre.is_empty() {
                        state.app_interface.version(&newest_version.to_string());
                    }
                    info!(
                        "[UPDATER] Version {} in use, {} is newest.",
                        app_version, newest_version
                    )
                } else {
                    info!("[UPDATER] Version {} in use.", app_version)
                }

                Self::handle_ui_startup(state, ctx);
                EmulatorController::set_enabled(
                    &mut ctx.program_state.emulator,
                    &state.app_interface,
                    ctx.cli.emulator_enabled(),
                );
            }
            AppMessage::RunUpdater => match ctx.updater.run_installer() {
                Ok(_) => {
                    // Terminate self
                    state.installer_spawned = true
                }
                Err(e) => {
                    error!("[UPDATER] Downloading installer failed. Reason: {}", e);
                    state.app_interface.update_failed();
                }
            },
            AppMessage::UpdateConfig { new_config } => {
                *ctx.config = new_config;
                ctx.config.write();
                info!("[CONFIG] Settings saved.");
            }
            AppMessage::ForceTakeControl => {
                if let Some(client) = ctx.network.transfer_client.as_ref() {
                    if let Some(client_name) = ctx.network.clients.get_client_in_control() {
                        //Will send a loopback Payloads::TransferControl
                        client.take_control(client_name.clone())
                    }
                }
            }
            AppMessage::EmulatorRequestVars => {
                EmulatorController::request_vars(
                    &ctx.program_state.emulator,
                    &ctx.sim.definitions,
                    &state.app_interface,
                );
            }
            AppMessage::EmulatorAddVar { name } => {
                EmulatorController::add_var(
                    &mut ctx.program_state.emulator,
                    &ctx.sim.definitions,
                    &state.app_interface,
                    &name,
                );
            }
            AppMessage::EmulatorRemoveVar { name } => {
                EmulatorController::remove_var(&mut ctx.program_state.emulator, &name);
            }
            AppMessage::EmulatorSetVar { name, value } => {
                let mut set_ctx = EmulatorSetContext {
                    definitions: &mut ctx.sim.definitions,
                    conn: &ctx.sim.conn,
                    client: ctx.network.transfer_client.as_deref(),
                    app: &state.app_interface,
                };
                EmulatorController::set_var(
                    &ctx.program_state.emulator,
                    &mut set_ctx,
                    &name,
                    value,
                );
            }
        }
    }

    fn load_definitions(state: &mut AppState, ctx: &mut AppContext<'_>) -> bool {
        let definition_path = &state.definitions_to_load;

        match ctx.sim.definitions.load_config(definition_path.clone()) {
            Ok(_) => {
                info!(
                    "[DEFINITIONS] Loaded and mapped {} aircraft vars, {} local vars, and {} events.",
                    ctx.sim.definitions.get_number_avars(),
                    ctx.sim.definitions.get_number_lvars(),
                    ctx.sim.definitions.get_number_events()
                );
            }
            Err(e) => {
                error!(
                    "[DEFINITIONS] Could not load configuration file {}: {}",
                    definition_path, e
                );
                return false;
            }
        };

        info!("[DEFINITIONS] {} loaded successfully", definition_path);

        // EmulatorController::send_vars_if_enabled(
        //     &ctx.program_state.emulator,
        //     &ctx.sim.definitions,
        //     &state.app_interface,
        // );

        true
    }

    fn connect_to_sim(state: &mut AppState, ctx: &mut AppContext<'_>) -> bool {
        ctx.sim.definitions = Definitions::new(); // Clear definitions prevent old definitions being used if sim is restarted

        let connected = if ctx.cli.skip_sim_connect() {
            info!("[SIM] SimConnect connection skipped (cli).");
            true
        } else {
            ctx.sim.conn.connect("YourControls")
        };

        if connected {
            // Display not connected to server message
            info!("[SIM] Connected to SimConnect.");
        } else {
            // Display trying to connect message
            state
                .app_interface
                .error("Could not connect to SimConnect! Is the sim running?");
        };

        connected
    }

    fn handle_start_server(
        state: &mut AppState,
        ctx: &mut AppContext<'_>,
        server_params: &StartServerParameters,
    ) {
        let sim_connected = Self::connect_to_sim(state, ctx);

        if !sim_connected {
            state
                .app_interface
                .error("Could not connect to SimConnect! Server not started.");
            return;
        }

        let skip_sim_connect = ctx.cli.skip_sim_connect();

        if !Self::load_definitions(state, ctx) {
            state
                .app_interface
                .error("Failed to load definition file! Server not started.");
            return;
        }

        ctx.sim
            .definitions
            .on_connected(&ctx.sim.conn, skip_sim_connect)
            .ok();
        // Display attempting to start server
        state.app_interface.attempt();

        let name = ctx.config.name.clone();
        let conn_timeout = ctx.config.conn_timeout;
        let version = ctx.updater.get_version_string();

        match server_params.method {
            ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
                let mut server = Box::new(Server::new(name.clone(), version.clone(), conn_timeout));

                let result = match server_params.method {
                    ConnectionMethod::Direct => server.start(
                        server_params.is_ipv6,
                        ctx.config.port,
                        server_params.use_upnp,
                    ),
                    ConnectionMethod::CloudServer => {
                        server.start_with_hole_punching(server_params.is_ipv6)
                    }
                    _ => panic!("Not implemented!"),
                };

                match result {
                    Ok(_) => {
                        // Assign server as transfer client
                        ctx.network.transfer_client = Some(server);
                        info!("[NETWORK] Server started.");
                    }
                    Err(e) => {
                        state.app_interface.server_fail(&e.to_string());
                        info!("[NETWORK] Could not start server! Reason: {}", e);
                    }
                }
            }
            ConnectionMethod::Relay => {
                let mut client = Box::new(Client::new(name, version, conn_timeout));

                match client.start_with_relay(server_params.is_ipv6) {
                    Ok(_) => {
                        ctx.network.transfer_client = Some(client);
                        info!("[NETWORK] Hosting started.");
                    }
                    Err(e) => {
                        info!("[NETWORK] Hosting could not start! Reason: {}", e);
                        state.app_interface.server_fail(&e.to_string());
                    }
                }
            }
        };

        ctx.config.write();
    }

    fn handle_auto_start(state: &mut AppState, ctx: &mut AppContext<'_>) {
        if !ctx.cli.start_server() || ctx.program_state.auto_start_pending {
            return;
        }

        ctx.program_state.auto_start_pending = true;

        let params = StartServerParameters {
            method: ctx.cli.connection_method(),
            is_ipv6: false, // TODO: Add CLI option for this
            use_upnp: true, // TODO: Add CLI option for this
        };

        Self::handle_start_server(state, ctx, &params);
    }

    fn handle_startup_load_definition(state: &mut AppState, ctx: &mut AppContext<'_>) {
        let Some(definition_file) = ctx.cli.definition_file() else {
            return;
        };

        let definition_file = definition_file.to_string();

        info!(
            "[STARTUP] Attempting to load definition file on startup: {}",
            definition_file
        );

        state.app_interface.set_aircraft(&definition_file);
    }

    fn handle_ui_startup(state: &mut AppState, ctx: &mut AppContext<'_>) {
        Self::handle_startup_load_definition(state, ctx);
        Self::handle_auto_start(state, ctx);
    }
}
