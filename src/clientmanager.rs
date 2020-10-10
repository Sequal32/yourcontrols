use std::{collections::HashMap};

#[derive(Default)]
pub struct Client {
    pub observer_mode: bool,
}

pub struct ClientManager {
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
        if let Some(name) = self.current_control.as_ref() {
            return self.clients.get(name)
        }
        return None
    }

    pub fn client_has_control(&self, name: &str) -> bool {
        if let Some(client_name) = self.current_control.as_ref() {
            return name == client_name
        }
        return false;
    }

    pub fn set_client_control(&mut self, name: String) -> Option<String> {
        if self.clients.contains_key(&name) {
            let previous_name = self.current_control.take();

            self.current_control = Some(name);

            return previous_name;
        }
        return None
    }

    pub fn set_no_control(&mut self) {
        self.current_control = None;
    }

    pub fn add_client(&mut self, name: String) {
        self.clients.insert(name.to_string(), Default::default());
    }

    pub fn remove_client(&mut self, name: &str) {
        self.clients.remove(name);
    }

    pub fn is_observer(&self, name: &str) -> bool {
        if let Some(client) = self.clients.get(name) {
            return client.observer_mode
        }
        return false
    }

    pub fn set_observer(&mut self, name: &str, is_observer: bool) {
        if let Some(client) = self.clients.get_mut(name) {
            client.observer_mode = is_observer;
        }
    }

    pub fn reset(&mut self) {
        self.clients.clear();
        self.current_control = None;
    }
}