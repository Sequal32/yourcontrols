use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    time::Instant,
};

use crate::util::{get_random_id, SESSION_ID_LENGTH};

pub struct Client {
    pub addr: SocketAddr,
    pub is_observer: bool,
}

pub struct ServerState {
    pub clients: HashMap<String, Client>,
    pub in_control: String,
    pub heartbeat_instant: Instant,
    pub started_at: Instant,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            in_control: "SERVER".to_string(),
            heartbeat_instant: Instant::now(),
            started_at: Instant::now(),
        }
    }
}

pub struct ServerInfo {
    pub creation_time: Instant,
    pub address: SocketAddr,
    pub addr_who_requested: SocketAddr,
}

impl ServerInfo {
    pub fn new(address: SocketAddr, addr_who_requested: SocketAddr) -> Self {
        Self {
            creation_time: Instant::now(),
            address,
            addr_who_requested,
        }
    }
}

pub struct StaticState {
    pub unknown_clients: HashMap<IpAddr, String>,
    pub clients_connected: HashMap<SocketAddr, String>,
    pub active_servers: HashMap<String, ServerInfo>,
}

impl StaticState {
    pub fn new() -> Self {
        Self {
            unknown_clients: HashMap::new(),
            clients_connected: HashMap::new(),
            active_servers: HashMap::new(),
        }
    }
}

pub struct Servers {
    pub meta_state: StaticState,
    pub server_states: HashMap<String, ServerState>,
}

impl Servers {
    pub fn new() -> Self {
        Self {
            meta_state: StaticState::new(),
            server_states: HashMap::new(),
        }
    }

    pub fn reserve_server(
        &mut self,
        address: SocketAddr,
        addr_who_requested: SocketAddr,
    ) -> String {
        let id = get_random_id(SESSION_ID_LENGTH);

        self.meta_state
            .active_servers
            .insert(id.clone(), ServerInfo::new(address, addr_who_requested));

        self.meta_state
            .clients_connected
            .insert(addr_who_requested, id.clone());

        self.server_states.insert(id.clone(), ServerState::new());

        id
    }

    pub fn remove_server(&mut self, session_id: &String) {
        self.meta_state.active_servers.remove(session_id);
        self.server_states.remove(session_id);
    }
}
