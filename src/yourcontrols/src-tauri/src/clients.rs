use std::collections::HashMap;
use yourcontrols_net::ControlDelegationsMap;
use yourcontrols_types::{ClientId, ControlSurfaces};

#[derive(Default)]
pub struct ClientInfo {
    name: String,
    id: ClientId,
    is_observer: bool,
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

    /// Whether the client is an observer
    pub fn is_observer(&self) -> bool {
        self.is_observer
    }

    /// Set the client info's is observer.
    pub fn set_observer(&mut self, is_observer: bool) {
        self.is_observer = is_observer;
    }

    /// Get a reference to the client info's id.
    pub fn id(&self) -> &ClientId {
        &self.id
    }

    /// Set the client info's id.
    pub fn set_id(&mut self, id: ClientId) {
        self.id = id;
    }
}

pub struct Clients {
    client_map: HashMap<ClientId, ClientInfo>,
    self_client: ClientInfo,
    delegations: ControlDelegationsMap,
    host: ClientId,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            client_map: HashMap::new(),
            self_client: ClientInfo::default(),
            host: u32::MAX,
            delegations: ControlDelegationsMap::new(),
        }
    }

    /// Get a reference to the client with the specified id
    pub fn get_client(&mut self, client_id: &ClientId) -> Option<&ClientInfo> {
        self.client_map.get(client_id)
    }

    /// Get a mutating reference to the client with the specified id
    pub fn get_client_mut(&mut self, client_id: &ClientId) -> Option<&mut ClientInfo> {
        self.client_map.get_mut(client_id)
    }

    pub fn self_client(&mut self) -> &mut ClientInfo {
        &mut self.self_client
    }

    pub fn add_client(&mut self, client_id: ClientId, name: String, is_observer: bool) {
        self.client_map.insert(
            client_id,
            ClientInfo {
                id: client_id,
                name,
                is_observer,
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

    /// Get all ControlSurfaces that the client defined by client_id currently has control of
    pub fn get_control_delegations_for_client(&self, client_id: &ClientId) -> Vec<ControlSurfaces> {
        self.delegations
            .iter()
            .filter_map(|(control_surface, client_delegated_to)| {
                if client_delegated_to == client_id {
                    Some(control_surface.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
