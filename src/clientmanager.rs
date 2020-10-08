use std::{collections::HashMap};

#[derive(Default)]
pub struct Client {
    pub observer_mode: bool,
    pub name: String
}

pub struct ClientManager<> {
    clients: HashMap<String, Client>,
    current_control: Option<String>
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            current_control: None
        }
    }

    // App could be the one in control
    pub fn who_has_control(&self) -> Option<&Client> {
        if let Some(ip) = self.current_control.as_ref() {
            return self.clients.get(ip)
        }
        return None
    }

    pub fn client_has_control(&self, ip: &str) -> bool {
        if let Some(client_name) = self.current_control.as_ref() {
            return ip == client_name
        }
        return false;
    }

    pub fn set_client_control(&mut self, ip: String) {
        if self.clients.contains_key(&ip) {
            self.current_control = Some(ip);
        }
    }

    pub fn set_no_control(&mut self) {
        self.current_control = None;
    }

    pub fn set_client_name(&mut self, ip: &str, name: String) {
        if let Some(client) = self.clients.get_mut(ip) {
            client.name = name.to_string();
        }
    }

    pub fn add_client(&mut self, ip: String) {
        self.clients.insert(ip.to_string(), Default::default());
    }

    pub fn remove_client(&mut self, ip: &str) {
        self.clients.remove(ip);
    }

    pub fn is_observer(&self, ip: &str) -> bool {
        if let Some(client) = self.clients.get(ip) {
            return client.observer_mode
        }
        return false
    }

    pub fn set_observer(&mut self, ip: &str, is_observer: bool) {
        if let Some(client) = self.clients.get_mut(ip) {
            client.observer_mode = is_observer;
        }
    }

    pub fn lookup_ip_from_name(&self, name: &str) -> Option<String> {
        for (ip, client) in self.clients.iter() {
            if name == client.name {return Some(ip.to_string())}
        }
        return None
    }
}