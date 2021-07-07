use std::collections::{hash_map::Values, HashMap};
use std::net::SocketAddr;
use yourcontrols_net::{ControlDelegationsMap, ControlSurfaces, MainPayloads};
use yourcontrols_types::ClientId;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub name: String,
    pub addr: SocketAddr,
    pub is_observer: bool,
    pub is_host: bool,
    pub id: ClientId,
}

impl ClientInfo {
    pub fn new(addr: SocketAddr, id: ClientId) -> Self {
        Self {
            name: String::new(),
            addr,
            id,
            is_observer: false,
            is_host: false,
        }
    }

    pub fn into_connected_payload(&self) -> MainPayloads {
        MainPayloads::ClientAdded {
            id: self.id,
            is_observer: self.is_observer,
            is_host: self.is_host,
            name: self.name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlDelegations {
    map: ControlDelegationsMap,
}

impl ControlDelegations {
    pub fn new() -> Self {
        Self {
            map: ControlDelegationsMap::new(),
        }
    }

    pub fn delegate(&mut self, control: ControlSurfaces, to: ClientId) {
        self.map.insert(control, to);
    }

    pub fn set_empty_control_delegations_to(&mut self, client: ClientId) {
        for surface in ControlSurfaces::all() {
            if self.map.get(&surface).is_none() {
                self.map.insert(surface, client);
            }
        }
    }

    pub fn remove_delegations(&mut self, to: &ClientId) {
        self.map
            .retain(|_, other_id| if other_id == to { false } else { true });
    }

    pub fn get_current_delegated_client(&self, control: &ControlSurfaces) -> Option<&ClientId> {
        self.map.get(control)
    }

    pub fn inner(&self) -> &ControlDelegationsMap {
        &self.map
    }
}

pub struct Clients {
    client_map: HashMap<ClientId, ClientInfo>,
    control_delegations: ControlDelegations,
    id_incrementer: u32,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            client_map: HashMap::new(),
            control_delegations: ControlDelegations::new(),
            id_incrementer: 0,
        }
    }

    pub fn all_addresses_except(&self, addr: &SocketAddr) -> Vec<SocketAddr> {
        self.client_map
            .values()
            .filter_map(|x| if x.addr == *addr { None } else { Some(x.addr) })
            .collect()
    }

    pub fn all_addresses(&self) -> Vec<SocketAddr> {
        self.client_map.values().map(|x| x.addr).collect()
    }

    pub fn get_address_for(&self, client: &ClientId) -> Option<SocketAddr> {
        self.client_map.get(client).map(|x| x.addr)
    }

    pub fn get(&self, client: &ClientId) -> Option<&ClientInfo> {
        self.client_map.get(client)
    }

    pub fn add(&mut self, addr: SocketAddr, id: Option<ClientId>) -> u32 {
        let id = id.unwrap_or_else(|| self.id_incrementer.wrapping_add(1));

        self.client_map.insert(id, ClientInfo::new(addr, id));

        return id;
    }

    pub fn remove(&mut self, id: &ClientId) -> Option<ClientInfo> {
        let removed_client = self.client_map.remove(id);

        // Cleanup delegations to the removing client
        self.control_delegations.remove_delegations(id);
        if let Some(host_id) = self.get_host().map(|x| x.id) {
            self.control_delegations
                .set_empty_control_delegations_to(host_id);
        }

        return removed_client;
    }

    pub fn remove_from_addr(&mut self, addr: &SocketAddr) -> Option<ClientInfo> {
        if let Some(client_id) = self
            .client_map
            .values()
            .find(|x| x.addr != *addr)
            .map(|x| x.id)
        {
            return self.remove(&client_id);
        }
        None
    }

    pub fn all(&self) -> Values<ClientId, ClientInfo> {
        self.client_map.values()
    }

    pub fn len(&self) -> usize {
        self.client_map.len()
    }

    pub fn make_observer(&mut self, client: &ClientId, observing: bool) -> Option<()> {
        self.client_map.get_mut(client)?.is_observer = observing;
        Some(())
    }

    pub fn make_first_host(&mut self) -> Option<&ClientInfo> {
        let client = self.client_map.values_mut().next()?;
        client.is_host = true;

        self.control_delegations
            .set_empty_control_delegations_to(client.id);

        Some(client)
    }

    pub fn set_name(&mut self, client: &ClientId, name: String) -> Option<()> {
        self.client_map.get_mut(client)?.name = name;
        None
    }

    pub fn get_host(&self) -> Option<&ClientInfo> {
        self.client_map.values().find(|x| x.is_host).map(|x| x)
    }

    pub fn get_id_for(&self, addr: SocketAddr) -> Option<ClientId> {
        self.client_map
            .values()
            .find(|x| x.addr == addr)
            .map(|x| x.id)
    }

    pub fn delegate(&mut self, control: ControlSurfaces, to: ClientId) {
        self.control_delegations.delegate(control, to);
    }

    pub fn delegations(&self) -> &ControlDelegationsMap {
        self.control_delegations.inner()
    }

    pub fn get_current_delegated_client(&self, control: &ControlSurfaces) -> Option<&ClientId> {
        self.control_delegations
            .get_current_delegated_client(control)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // #[test]
}
