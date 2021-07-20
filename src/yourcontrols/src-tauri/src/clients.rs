use std::collections::HashMap;
use yourcontrols_net::ControlDelegationsMap;
use yourcontrols_types::{ClientId, ControlSurfaces};

#[derive(Default)]
pub struct ClientInfo {
    id: ClientId,
    name: String,
    is_observer: bool,
}

impl ClientInfo {
    pub fn new(id: ClientId, name: String, is_observer: bool) -> Self {
        Self {
            id,
            name,
            is_observer,
        }
    }

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
        self.client_map
            .insert(client_id, ClientInfo::new(client_id, name, is_observer));
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

    /// Returns true if the client currently has control of the specified delegation
    pub fn client_has_delegation(
        &self,
        client_id: &ClientId,
        delegation: &ControlSurfaces,
    ) -> bool {
        self.delegations
            .get(delegation)
            .map(|client_with_delegation| client_with_delegation == client_id)
            .unwrap_or(false)
    }

    /// Returns true if the self_client currently has control of the specified delegation
    pub fn self_has_delegation(&self, delegation: &ControlSurfaces) -> bool {
        self.client_has_delegation(self.self_client.id(), delegation)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_SELF_ID: u32 = 0;
    const TEST_CLIENT2_ID: u32 = 1;

    fn get_test_clients() -> Clients {
        let mut clients = Clients::new();

        *clients.self_client() = ClientInfo::new(TEST_SELF_ID, "TEST_CLIENT1".to_string(), false);
        clients.add_client(TEST_CLIENT2_ID, "TEST_CLIENT2".to_string(), false);

        return clients;
    }

    #[test]
    fn test_set_host() {
        let mut clients = get_test_clients();
        clients.set_host(0);
        assert!(clients.is_host(&0));
    }

    #[test]
    fn test_control_delegations() {
        let mut delegations = HashMap::new();
        delegations.insert(ControlSurfaces::Yoke, TEST_SELF_ID);
        delegations.insert(ControlSurfaces::Throttle, TEST_CLIENT2_ID);

        let mut clients = get_test_clients();
        clients.set_control_delegations(delegations);

        assert!(clients.client_has_delegation(&TEST_CLIENT2_ID, &ControlSurfaces::Throttle));
        assert!(!clients.client_has_delegation(&TEST_CLIENT2_ID, &ControlSurfaces::Yoke));
        // Undelegated
        assert!(!clients.client_has_delegation(&TEST_CLIENT2_ID, &ControlSurfaces::Mixture));

        assert!(clients.self_has_delegation(&ControlSurfaces::Yoke));
        assert!(!clients.self_has_delegation(&ControlSurfaces::Throttle));
        // Undelegated
        assert!(!clients.self_has_delegation(&ControlSurfaces::Mixture));
    }
}
