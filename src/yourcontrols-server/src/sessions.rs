use crate::util::get_random_id;
use bimap::BiHashMap;
use std::{collections::HashMap, net::SocketAddr};

pub struct Sessions {
    hosting_sessions: BiHashMap<SocketAddr, String>,
    connected_sessions: HashMap<SocketAddr, String>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            hosting_sessions: BiHashMap::new(),
            connected_sessions: HashMap::new(),
        }
    }

    pub fn map_session_id_to_socket_info(&mut self, addr: SocketAddr) -> String {
        let id = get_random_id(8);

        // Point session id to hosting IP
        self.hosting_sessions.insert(addr, id.clone());
        // Keep track of all ips in the session
        self.connected_sessions.insert(addr, id.clone());

        id
    }

    pub fn get_socket_info_for_session(&self, session_id: &String) -> Option<&SocketAddr> {
        self.hosting_sessions.get_by_right(session_id)
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
    pub fn close_session(&mut self, session_id: &String) {
        self.hosting_sessions.remove_by_right(session_id);
        self.connected_sessions
            .retain(|_, connected_session_id| connected_session_id != session_id);
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
