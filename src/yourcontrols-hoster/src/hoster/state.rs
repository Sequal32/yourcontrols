use std::{collections::HashMap, net::SocketAddr, time::Instant};

use yourcontrols_net::{Payloads, SenderReceiver};

pub const SERVER_NAME: &str = "SERVER";

pub struct ClientConnection {
    pub addr: SocketAddr,
    pub is_observer: bool,
}

pub struct ServerState {
    pub clients: HashMap<String, ClientConnection>,
    pub in_control: String,
    pub hoster: String,
    pub heartbeat_instant: Instant,
    pub created_at: Instant,
}

#[allow(dead_code)]
impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            in_control: SERVER_NAME.to_string(),
            hoster: SERVER_NAME.to_string(),
            heartbeat_instant: Instant::now(),
            created_at: Instant::now(),
        }
    }

    pub fn get_from_addr(&mut self, addr: SocketAddr) -> Option<&mut ClientConnection> {
        self.clients
            .iter_mut()
            .map(|(_, client)| client)
            .find(|client| client.addr == addr)
    }

    pub fn add_client(&mut self, name: String, addr: SocketAddr, is_observer: bool) {
        self.clients
            .insert(name, ClientConnection { addr, is_observer });
    }

    pub fn remove_client(&mut self, name: &str) {
        self.clients.remove(name);
    }

    pub fn remove_client_by_addr(&mut self, addr: &SocketAddr) -> Option<String> {
        let mut removed_name: Option<String> = None;

        self.clients.retain(|name, client| {
            if client.addr != *addr {
                true
            } else {
                removed_name = Some(name.clone());
                false
            }
        });

        removed_name
    }

    pub fn send_to_all(
        &mut self,
        payload: Payloads,
        except: Option<&SocketAddr>,
        net: &mut SenderReceiver,
    ) {
        let mut to_send = Vec::new();

        for (_, client) in self.clients.iter() {
            if let Some(except) = except {
                if client.addr == *except {
                    continue;
                }
            }

            to_send.push(client.addr);
        }

        net.send_message_to_multiple(payload, to_send).ok();
    }

    pub fn set_host(&mut self, name: String, net: &mut SenderReceiver) {
        let client = self.clients.get_mut(&name).expect("always there");
        client.is_observer = false;

        net.send_message(Payloads::SetHost, client.addr).ok();
        self.send_to_all(
            Payloads::TransferControl {
                from: self.in_control.clone(),
                to: name.clone(),
            },
            None,
            net,
        );

        self.in_control = name.clone();
        self.hoster = name;
    }

    pub fn process_payload(
        &mut self,
        addr: SocketAddr,
        payload: Payloads,
        net: &mut SenderReceiver,
    ) {
        match &payload {
            // Unused
            Payloads::InvalidName { .. }
            | Payloads::RendezvousHandshake { .. }
            | Payloads::InvalidVersion { .. }
            | Payloads::PlayerJoined { .. }
            | Payloads::HostingReceived { .. }
            | Payloads::SetHost { .. }
            | Payloads::AttemptConnection { .. }
            | Payloads::AttemptHosterConnection { .. }
            | Payloads::RequestHosting { .. }
            | Payloads::PeerEstablished { .. }
            | Payloads::ConnectionDenied { .. }
            | Payloads::Heartbeat
            | Payloads::SetSelfObserver { .. }
            | Payloads::PlayerLeft { .. } => return,
            // Used
            Payloads::AircraftDefinition { .. } | Payloads::Update { .. } => {}
            Payloads::InitHandshake { name, version } => {
                let server_version = dotenv::var("APP_VERSION").unwrap();

                if *version != server_version {
                    net.send_message(Payloads::InvalidVersion { server_version }, addr)
                        .ok();
                    return;
                }

                if self.clients.contains_key(name) {
                    net.send_message(Payloads::InvalidName {}, addr).ok();
                    return;
                }

                // Send all current connected clients
                for (name, info) in self.clients.iter() {
                    net.send_message(
                        Payloads::PlayerJoined {
                            name: name.clone(),
                            in_control: self.in_control == *name,
                            is_server: self.hoster == *name,
                            is_observer: info.is_observer,
                        },
                        addr,
                    )
                    .ok();
                }

                // Add client
                self.add_client(name.clone(), addr, true);

                // If the client is the first one to connect, give them control and have them "host"
                if self.in_control == SERVER_NAME {
                    self.set_host(name.clone(), net);
                }

                self.send_to_all(
                    Payloads::PlayerJoined {
                        name: name.clone(),
                        in_control: false,
                        is_server: false,
                        is_observer: true,
                    },
                    Some(&addr),
                    net,
                );

                return;
            }
            Payloads::TransferControl { to, .. } => {
                self.in_control = to.clone();
            }
            Payloads::SetObserver {
                to, is_observer, ..
            } => {
                if let Some(client) = self.clients.get_mut(to) {
                    client.is_observer = *is_observer;
                }
            }
            Payloads::Ready => {
                // Tell "host" to do a full sync
                if let Some(client) = self.clients.get(&self.in_control) {
                    net.send_message(payload, client.addr).ok();
                }

                return;
            }
            Payloads::Handshake { .. } => {
                net.send_message(payload, addr).ok();

                return;
            }
        }

        self.send_to_all(payload, Some(&addr), net);
    }
}

pub struct ActiveState {
    clients_connected: HashMap<SocketAddr, String>,
    server_states: HashMap<String, ServerState>,
}

#[allow(dead_code)]
impl ActiveState {
    pub fn new() -> Self {
        Self {
            clients_connected: HashMap::new(),
            server_states: HashMap::new(),
        }
    }

    pub fn add_server(&mut self, session_id: String) {
        self.server_states.insert(session_id, ServerState::new());
    }

    pub fn remove_server(&mut self, session_id: &str) {
        self.server_states.remove(session_id);
    }

    pub fn add_client(&mut self, addr: SocketAddr, session_id: String) {
        self.clients_connected.insert(addr, session_id);
    }

    pub fn remove_client(&mut self, addr: &SocketAddr) -> Option<String> {
        self.clients_connected.remove(addr)
    }

    pub fn get_session_id_for(&self, addr: &SocketAddr) -> Option<&String> {
        self.clients_connected.get(addr)
    }

    pub fn get_server_state(&mut self, session_id: &str) -> Option<&mut ServerState> {
        self.server_states.get_mut(session_id)
    }

    pub fn get_server_state_for(&mut self, addr: &SocketAddr) -> Option<&mut ServerState> {
        self.server_states
            .get_mut(self.clients_connected.get(addr)?)
    }

    pub fn get_server_states(&mut self) -> &mut HashMap<String, ServerState> {
        &mut self.server_states
    }

    pub fn remove_unused(&mut self) -> Vec<String> {
        let mut removed = Vec::new();

        self.server_states.retain(|session_id, state| {
            if !state.clients.is_empty() && state.created_at.elapsed().as_secs() > 60 {
                removed.push(session_id.clone());
                false
            } else {
                true
            }
        });

        removed
    }
}
