use std::collections::HashMap;
use yourcontrols_net::ControlDelegationsMap;
use yourcontrols_types::ClientId;

#[derive(Default)]
pub struct ClientInfo {
    pub name: String,
    pub is_host: bool,
    pub is_observer: bool,
}

impl ClientInfo {
    /// Get a reference to the client info's name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Set the client info's name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

pub struct Clients {
    client_map: HashMap<ClientId, ClientInfo>,
    delegations: ControlDelegationsMap,
    host: ClientId,
    self_id: ClientId,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            client_map: HashMap::new(),
            self_id: u32::MAX,
            host: u32::MAX,
            delegations: ControlDelegationsMap::new(),
        }
    }

    pub fn get_name(&self, client_id: &ClientId) -> Option<&String> {
        self.client_map.get(client_id).map(ClientInfo::name)
    }

    pub fn set_name(&mut self, client_id: &ClientId, name: String) {
        if let Some(client) = self.client_map.get_mut(client_id) {
            client.set_name(name)
        }
    }

    pub fn add_client(
        &mut self,
        client_id: ClientId,
        name: String,
        is_host: bool,
        is_observer: bool,
    ) {
        self.client_map.insert(
            client_id,
            ClientInfo {
                name,
                is_host,
                is_observer,
                ..Default::default()
            },
        );
    }

    pub fn remove_client(&mut self, client_id: &ClientId) {
        self.client_map.remove(client_id);
    }

    pub fn is_host(&mut self, client_id: &ClientId) -> bool {
        self.host == *client_id
    }

    pub fn set_host(&mut self, client_id: ClientId) {
        self.host = client_id
    }

    pub fn set_control_delegations(&mut self, delegations: ControlDelegationsMap) {
        self.delegations = delegations;
    }

    pub fn set_self_id(&mut self, client_id: ClientId) {
        self.self_id = client_id;
    }

    pub fn self_id(&self) -> &ClientId {
        &self.self_id
    }
}
