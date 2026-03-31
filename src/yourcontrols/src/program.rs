use std::{
    default,
    path::{Path, PathBuf},
};

use log::{error, info};
use serde::de;
use simconnect::SimConnector;
use yourcontrols_net::{Client, Server, TransferClient};

use crate::{
    app::{App, ConnectionMethod},
    definitions::Definitions,
    simconfig::Config,
    sync::control::Control,
    update::Updater,
};

/// Parameters for starting the server
pub struct StartServerParameters {
    pub method: ConnectionMethod,
    pub is_ipv6: bool,
    pub use_upnp: bool,
}

// Temporary implementation of the whole program handler. All mutable references so this can be worked on incrementally.
pub struct Program<'a> {
    pub conn: &'a mut SimConnector,
    pub definitions: &'a mut Definitions,
    pub config: &'a mut Config,
    pub transfer_client: &'a mut Option<Box<dyn TransferClient>>,
    pub updater: &'a Updater,
    pub control: &'a mut Control,
    pub app_interface: &'a App,
    pub cli: &'a crate::cli::CliWrapper,
    pub state: &'a mut ProgramState,
}

#[derive(Default)]
pub struct ProgramState {
    auto_start_pending: bool,
}

impl<'a> Program<'a> {
    /// Loads the definitions for the selected sim and config, returns whether the load was successful.
    pub fn load_definitions(&mut self, definition_path: &Path) -> bool {
        match self
            .definitions
            .load_config(definition_path.to_string_lossy().to_string())
        {
            Ok(_) => {
                info!("[DEFINITIONS] Loaded and mapped {} aircraft vars, {} local vars, and {} events.",
                self.definitions.get_number_avars(), self.definitions.get_number_lvars(), self.definitions.get_number_events());
            }
            Err(e) => {
                error!(
                    "[DEFINITIONS] Could not load configuration file {}: {}",
                    definition_path.display(),
                    e
                );
                return false;
            }
        };

        info!(
            "[DEFINITIONS] {} loaded successfully",
            definition_path.display()
        );

        true
    }

    /// Connects to SimConnect, returns whether the connection was successful.
    pub fn connect_to_sim(&mut self) -> bool {
        // Connect to simconnect
        *self.definitions = Definitions::new();

        let connected = if self.cli.skip_sim_connect() {
            info!("[SIM] SimConnect connection skipped (cli).");
            true
        } else {
            self.conn.connect("YourControls")
        };

        if connected {
            // Display not connected to server message
            info!("[SIM] Connected to SimConnect.");
        } else {
            // Display trying to connect message
            self.app_interface
                .error("Could not connect to SimConnect! Is the sim running?");
        };

        connected
    }

    /// Handles starting a server that clients can connect to, including connecting to SimConnect, loading definitions, and starting the server.
    pub fn handle_start_server(&mut self, server_params: &StartServerParameters) {
        let sim_connected = self.connect_to_sim();

        if !sim_connected {
            self.app_interface
                .error("Could not connect to SimConnect! Server not started.");
            return;
        }

        self.definitions
            .on_connected(self.conn, self.cli.skip_sim_connect())
            .ok();
        self.control.on_connected(self.conn);
        // Display attempting to start server
        self.app_interface.attempt();

        match server_params.method {
            ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
                let mut server = Box::new(Server::new(
                    self.config.name.clone(),
                    self.updater.get_version_string(),
                    self.config.conn_timeout,
                ));

                let result = match server_params.method {
                    ConnectionMethod::Direct => server.start(
                        server_params.is_ipv6,
                        self.config.port,
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
                        *self.transfer_client = Some(server);
                        info!("[NETWORK] Server started.");
                    }
                    Err(e) => {
                        self.app_interface.server_fail(&e.to_string());
                        info!("[NETWORK] Could not start server! Reason: {}", e);
                    }
                }
            }
            ConnectionMethod::Relay => {
                let mut client = Box::new(Client::new(
                    self.config.name.clone(),
                    self.updater.get_version_string(),
                    self.config.conn_timeout,
                ));

                match client.start_with_relay(server_params.is_ipv6) {
                    Ok(_) => {
                        *self.transfer_client = Some(client);
                        info!("[NETWORK] Hosting started.");
                    }
                    Err(e) => {
                        info!("[NETWORK] Hosting could not start! Reason: {}", e);
                        self.app_interface.server_fail(&e.to_string());
                    }
                }
            }
        };

        self.config.write();
    }

    fn handle_auto_start(&mut self) {
        if !self.cli.start_server() || self.state.auto_start_pending {
            return;
        }

        self.state.auto_start_pending = true;

        let params = StartServerParameters {
            method: self.cli.connection_method(),
            is_ipv6: false, // TODO: Add CLI option for this
            use_upnp: true, // TODO: Add CLI option for this
        };

        self.handle_start_server(&params);
    }

    fn handle_startup_load_definition(&mut self) {
        let Some(definition_file) = self.cli.definition_file() else {
            return;
        };

        info!(
            "[STARTUP] Attempting to load definition file on startup: {}",
            definition_file
        );

        if self.load_definitions(&PathBuf::from(definition_file)) {
            self.app_interface.set_aircraft(definition_file);
        } else {
            self.app_interface.error(
                "Failed to load definition file on startup! Check the logs for more details.",
            );
        }
    }

    pub fn handle_ui_startup(&mut self) {
        self.handle_startup_load_definition();
        self.handle_auto_start();
    }
}
