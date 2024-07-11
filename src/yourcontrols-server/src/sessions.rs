use crate::util::{get_random_id, SESSION_ID_LENGTH};
use bimap::BiHashMap;
use std::{collections::HashMap, net::SocketAddr};

pub struct SessionInfo {
    pub hoster_addr: SocketAddr,
    pub hoster_endpoints: Vec<SocketAddr>,
}

impl SessionInfo {
    pub fn new(hoster_addr: SocketAddr, hoster_endpoints: Vec<SocketAddr>) -> Self {
        Self {
            hoster_addr,
            hoster_endpoints,
        }
    }
}

pub struct Sessions {
    hosting_sessions: BiHashMap<SocketAddr, String>,
    connected_sessions: HashMap<SocketAddr, String>,
    session_info: HashMap<String, SessionInfo>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            hosting_sessions: BiHashMap::new(),
            connected_sessions: HashMap::new(),
            session_info: HashMap::new(),
        }
    }

    pub fn map_session_id_to_socket_info(
        &mut self,
        addr: SocketAddr,
        endpoints: Vec<SocketAddr>,
    ) -> String {
        let id = get_random_id(SESSION_ID_LENGTH);

        // Point session id to hosting IP
        self.hosting_sessions.insert(addr, id.clone());
        // Point session id to all possible connectable endpoints
        self.session_info
            .insert(id.clone(), SessionInfo::new(addr, endpoints));

        id
    }

    pub fn get_session_connection_info(&self, session_id: &str) -> Option<&SessionInfo> {
        self.session_info.get(session_id)
    }

    pub fn add_client_to_session(&mut self, session_id: String, addr: SocketAddr) {
        self.connected_sessions.insert(addr, session_id);
    }

    pub fn remove_client_from_session(&mut self, addr: &SocketAddr) -> Option<String> {
        self.connected_sessions.remove(addr)
    }

    pub fn socket_is_hosting(&self, socket: &SocketAddr) -> bool {
        self.hosting_sessions.contains_left(socket)
    }

    // Returns socket info that were in that session
    pub fn close_session(&mut self, session_id: &str) {
        self.hosting_sessions.remove_by_right(session_id);
        self.connected_sessions
            .retain(|_, connected_session_id| connected_session_id != session_id);
        self.session_info.remove(session_id);
    }

    pub fn close_session_by_addr(&mut self, addr: &SocketAddr) -> Option<String> {
        if let Some(session_id) = self.hosting_sessions.get_by_left(addr).cloned() {
            self.close_session(&session_id);
            Some(session_id.clone())
        } else {
            None
        }
    }

    pub fn get_session_count(&self) -> usize {
        self.hosting_sessions.len()
    }

    pub fn get_user_count(&self) -> usize {
        self.connected_sessions.len()
    }
}
