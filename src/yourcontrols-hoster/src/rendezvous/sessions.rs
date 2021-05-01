use crate::util::{encode_ip, get_random_id};
use std::{collections::HashMap, net::SocketAddr};

pub struct EncodedSessionAddress {
    pub hoster: SocketAddr,
    pub private: String,
    pub public: String,
}

pub struct Sessions {
    hosting_sessions: HashMap<String, EncodedSessionAddress>,
    connected_sessions: HashMap<SocketAddr, String>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            hosting_sessions: HashMap::new(),
            connected_sessions: HashMap::new(),
        }
    }

    pub fn map_session_id_to_socket_info(&mut self, addr: SocketAddr) -> String {
        let id = get_random_id(5);

        let session_info = EncodedSessionAddress {
            hoster: addr,
            private: encode_ip(&addr),
            public: encode_ip(&addr),
        };
        // Point session id to hosting IP
        self.hosting_sessions.insert(id.clone(), session_info);
        // Keep track of all ips in the session
        self.connected_sessions.insert(addr.clone(), id.clone());

        return id;
    }

    pub fn get_info_for_session(&self, session_id: &String) -> Option<&EncodedSessionAddress> {
        self.hosting_sessions.get(session_id)
    }

    pub fn add_client_to_session(&mut self, session_id: String, addr: SocketAddr) {
        self.connected_sessions.insert(addr, session_id);
    }

    pub fn remove_client_from_session(&mut self, addr: &SocketAddr) -> Option<String> {
        self.connected_sessions.remove(addr)
    }

    pub fn socket_is_hosting(&self, socket: &SocketAddr) -> bool {
        todo!()
    }

    // Returns socket info that were in that session
    pub fn close_session(&mut self, session_id: &String) {
        self.hosting_sessions.remove(session_id);
        self.connected_sessions
            .retain(|_, connected_session_id| connected_session_id != session_id);
    }

    pub fn close_session_by_addr(&mut self, addr: &SocketAddr) -> Option<String> {
        // if let Some(session_id) = self.hosting_sessions.get_by_left(addr).cloned() {
        //     self.close_session(&session_id);
        //     Some(session_id.clone())
        // } else {
        //     None
        // }
        todo!()
    }

    pub fn get_session_count(&self) -> usize {
        self.hosting_sessions.len()
    }

    pub fn get_user_count(&self) -> usize {
        self.connected_sessions.len()
    }
}
