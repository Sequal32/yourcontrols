use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    time::Instant,
};

use crate::util::{get_random_id, SESSION_ID_LENGTH};

pub struct Client {
    pub addr: SocketAddr,
    pub is_observer: bool,
    pub is_host: bool,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            is_observer: false,
            is_host: false,
        }
    }
}

pub struct ServerState {
    pub clients: HashMap<String, Client>,
    pub aircraft_definition: Option<Box<[u8]>>,
    pub in_control: String,
    pub heartbeat_instant: Instant,
    pub started_at: Instant,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            in_control: "SERVER".to_string(),
            aircraft_definition: None,
            heartbeat_instant: Instant::now(),
            started_at: Instant::now(),
        }
    }
}

pub struct ServerInfo {
    pub hostname: String,
}

impl ServerInfo {
    pub fn new(hostname: String) -> Self {
        Self { hostname }
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

    pub fn reserve_server(&mut self, hostname: String, addr_who_requested: SocketAddr) -> String {
        let id = get_random_id(SESSION_ID_LENGTH);

        self.meta_state
            .active_servers
            .insert(id.clone(), ServerInfo::new(hostname));

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
