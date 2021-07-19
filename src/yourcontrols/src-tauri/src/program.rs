use anyhow::{ensure, Result};
use std::collections::HashMap;
use std::fs::read_dir;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use yourcontrols_net::MainPayloads;

use crate::aircraft::DefinitionsUpdater;
use crate::clients::Clients;
use crate::network::{Network, NetworkEvent};
use crate::simulator::Simulator;
use crate::ui::cmd::UiEvents;
use crate::ui::Ui;

use yourcontrols_definitions::DefinitionsParser;
use yourcontrols_types::{ChangedDatum, ControlSurfaces, Payloads, WatchPeriod};

const DEFINITIONS_PATH: &str = "definitions";

pub struct Program<U> {
    ui: U,
    definitions_parser: DefinitionsParser,
    definitions_updater: DefinitionsUpdater,
    network: Network,
    simulator: Simulator,
    clients: Clients,
}

impl<U: Ui> Program<U> {
    pub fn setup() -> Self {
        Self {
            ui: Ui::run(),
            network: Network::new(),
            definitions_parser: DefinitionsParser::new(),
            definitions_updater: DefinitionsUpdater::new(),
            simulator: Simulator::new(),
            clients: Clients::new(),
        }
    }

    /// Reads definitions from the filesystem and loads them into the definitions parser
    pub fn load_definitions(&mut self, target_definition_sub_path: impl AsRef<Path>) -> Result<()> {
        let mut core_path = PathBuf::from(DEFINITIONS_PATH);
        core_path.push("templates");

        for entry in read_dir(core_path)? {
            let entry = entry?;
            self.definitions_parser.load_file(entry.path())?
        }

        let mut scripts_path = PathBuf::from(DEFINITIONS_PATH);
        scripts_path.push("scripts");

        self.definitions_parser.load_scripts(scripts_path)?;

        let mut definition_path = PathBuf::from(DEFINITIONS_PATH);
        definition_path.push(target_definition_sub_path);

        self.definitions_parser.load_file(definition_path)?;

        Ok(())
    }

    /// Connects to the simulator and readies the definitions
    pub fn connect_to_simulator(&mut self) -> bool {
        let success = self.simulator.connect("YourControls");

        if !success {
            return false;
        }

        self.simulator.send_message(Payloads::SetMappings {
            datums: self.definitions_parser.get_parsed_datums().clone(),
            vars: self.definitions_parser.get_parsed_vars().clone(),
            events: self.definitions_parser.get_parsed_events().clone(),
            scripts: self.definitions_parser.get_parsed_scripts(),
        });

        return true;
    }

    /// Polls and processes events from the gauge
    pub fn process_sim_events(&mut self) -> Result<()> {
        let msg = match self.simulator.poll() {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };

        match msg {
            Payloads::VariableChange { changed } => {
                self.sim_on_vars_changed(changed)?;
            }
            _ => {}
        };

        Ok(())
    }

    /// Returns true if the datum is being updated every frame
    fn datum_is_frame(&self, datum: &ChangedDatum) -> bool {
        self.definitions_parser
            .get_parsed_datums()
            .get(datum.key)
            .map(|x| x.watch_period == Some(WatchPeriod::Frame))
            .unwrap_or(false)
    }

    /// Splits a vector of ChangedDatum into two based on whether its updating every frame
    fn split_reliable_unreliable(
        &self,
        changed: Vec<ChangedDatum>,
    ) -> (Vec<ChangedDatum>, Vec<ChangedDatum>) {
        changed
            .into_iter()
            .partition(|datum| self.datum_is_frame(datum))
    }

    /// Sends updated vars across the network
    fn sim_on_vars_changed(&mut self, changed: Vec<ChangedDatum>) -> Result<()> {
        let (unreliable, reliable) = self.split_reliable_unreliable(changed);

        self.network
            .send_update(*self.clients.self_client().id(), 0.0, unreliable, reliable)?;

        Ok(())
    }

    /// Polls and processes all network events
    pub fn process_network_events(&mut self) -> Result<()> {
        let events = match self.network.step() {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for event in events {
            println!("{:?}", event);
            match event {
                // Session Events
                NetworkEvent::Payload(MainPayloads::Welcome { client_id, name }) => {
                    self.on_welcome_received(client_id, name)?
                }
                NetworkEvent::Payload(MainPayloads::MakeHost { client_id }) => {
                    self.clients.set_host(client_id);
                }
                NetworkEvent::Payload(MainPayloads::ClientAdded {
                    client_id: id,
                    is_host,
                    is_observer,
                    name,
                }) => {
                    self.on_client_added_received(id, name, is_observer, is_host);
                }
                NetworkEvent::Payload(MainPayloads::ClientRemoved { client_id }) => {
                    self.on_client_removed_received(client_id);
                }
                NetworkEvent::Payload(MainPayloads::ControlDelegations { delegations }) => {
                    self.on_delegations_received(delegations);
                }
                NetworkEvent::SessionReceived { session_id } => {
                    self.on_session_received(session_id);
                }
                NetworkEvent::Connected => {
                    self.on_connected_received()?;
                }
                // Game Events
                NetworkEvent::Payload(MainPayloads::Update {
                    client_id,
                    changed,
                    time,
                    ..
                }) => {
                    self.on_update_received(client_id, changed, time);
                }

                _ => {}
            }
        }

        Ok(())
    }

    /// Verifies the server sent back a valid name and stores the assigned client_id
    fn on_welcome_received(&mut self, client_id: u32, name: String) -> Result<()> {
        ensure!(
            *self.clients.self_client().name() == name,
            "Bad server Welcome."
        );

        self.clients.self_client().set_id(client_id);

        Ok(())
    }

    /// Stores the added client's information and notifies the UI
    fn on_client_added_received(
        &mut self,
        id: u32,
        name: String,
        is_observer: bool,
        is_host: bool,
    ) {
        self.clients.add_client(id, name, is_observer);
        if is_host {
            self.clients.set_host(id);
        }
    }

    /// Removes the client's information and notifies the UI
    fn on_client_removed_received(&mut self, client_id: u32) {
        self.clients.remove_client(&client_id);
    }

    /// Stores the new control delegations
    fn on_delegations_received(&mut self, delegations: HashMap<ControlSurfaces, u32>) {
        self.clients.set_control_delegations(delegations);
    }

    /// Notifies the UI of the session_id
    fn on_session_received(&mut self, session_id: String) {
        self.ui.send_message(UiEvents::LobbyInfo {
            session_code: Some(session_id),
            server_ip: None,
            clients: None,
        });
    }

    /// Notifies the UI we sucessfully established a connection and sends our requested name to the server
    fn on_connected_received(&mut self) -> Result<(), anyhow::Error> {
        self.ui.send_message(UiEvents::Connected);

        self.network.send_payload_to_server(MainPayloads::Name {
            name: self.clients.self_client().name().clone(),
        })?;

        Ok(())
    }

    /// Verifies the client is sending legit data with their current permissions and sends the update to the gauge
    fn on_update_received(&mut self, client_id: u32, changed: Vec<ChangedDatum>, time: f64) {
        let should_allow_update = self
            .clients
            .get_client(&client_id)
            .map(|c| !c.is_observer())
            .unwrap_or(false);

        let delegated_controls = self.clients.get_control_delegations_for_client(&client_id);

        let definitions_match_client_delegations = changed.iter().all(|datum| {
            self.definitions_parser
                .get_meta_data_for(&datum.key)
                .and_then(|data| {
                    data.control_surface
                        .as_ref()
                        .map(|x| delegated_controls.contains(&x))
                })
                .unwrap_or(true)
        });

        if should_allow_update && definitions_match_client_delegations {
            self.simulator.send_message(Payloads::SendIncomingValues {
                data: changed,
                time, // TODO: pre-process time
            });
        }
    }

    /// Polls the UI for user-events and processes them
    pub fn process_ui_events(&mut self) -> Result<()> {
        let event = match self.ui.next_event() {
            Some(e) => e,
            None => return Ok(()),
        };

        match event {
            UiEvents::UiReady => {
                self.ui_on_ready();
            }
            UiEvents::InstallAircraft { names } => {}
            UiEvents::TestNetwork { port } => {}
            UiEvents::Join {
                port,
                session_code,
                server_ip,
                username,
            } => {
                self.ui_on_join_request(username, port, server_ip, session_code)?;
            }
            UiEvents::Host { port, username } => {
                self.ui_on_host_request(port, username)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// The UI is ready to accept data - send them all the data neccessary to fully startup
    fn ui_on_ready(&mut self) {
        self.ui.send_message(UiEvents::StartUpText {
            text: "Starting...".to_string(),
        });

        self.ui.send_message(UiEvents::InitData {
            version: std::env::var("CARGO_PKG_VERSION").unwrap(),
            aircraft: self.definitions_updater.get_all_aircraft_info(),
        });

        self.ui.send_message(UiEvents::LoadingComplete);
    }

    /// Connect directly to the server_ip/port if not None, otherwise connect using the session_code provided
    fn ui_on_join_request(
        &mut self,
        username: String,
        port: Option<u16>,
        server_ip: Option<String>,
        session_code: Option<String>,
    ) -> Result<()> {
        self.clients.self_client().set_name(username);

        let addr = server_ip
            .and_then(|ip_string| ip_string.parse::<IpAddr>().ok())
            .map(|ip| SocketAddr::new(ip, port.expect("should always be sent with ip")));

        if let Some(addr) = addr {
            self.network.connect_to_address(addr)?
        } else if let Some(session_code) = session_code {
            self.network.connect_to_session(session_code)?
        }

        Ok(())
    }

    /// Host directly if port is None, otherwise request a session_id and begin hosting
    fn ui_on_host_request(&mut self, port: Option<u16>, username: String) -> Result<()> {
        self.clients.self_client().set_name(username);

        Ok(if let Some(port) = port {
            self.network.start_direct(port)?;
        } else {
            self.network.start_cloud_p2p()?;
        })
    }

    /// The main loop
    pub fn poll(&mut self) -> Result<()> {
        self.process_ui_events()?;
        self.process_network_events()?;
        self.process_sim_events()?;

        Ok(())
    }
}
