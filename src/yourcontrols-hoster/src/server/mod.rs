mod clients;
#[cfg(test)]
mod test;

use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::{bail, Result};
use clients::Clients;
use yourcontrols_net::{BaseSocket, MainPayloads, Message, StartableNetworkObject};
use yourcontrols_types::{ClientId, ControlSurfaces};

use self::clients::ClientInfo;

/// A server for communication between YourControls clients
pub struct SingleServer {
    // Manage state between clients
    clients: Clients,
    // The socket used to communicate with all clients
    socket: BaseSocket,
    // Whether the server can accept new clients
    // used for when we're still waiting for a session_id
    can_accept_clients: bool,
    // The received session_id when requesting one from the rendezvous server
    session_id: Option<String>,
    // Helper metadata for communication
    metadata: ServerMetadata,
}

impl StartableNetworkObject<anyhow::Error> for SingleServer {
    fn start_with_bind_address(address: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self {
            clients: Clients::new(),
            socket: BaseSocket::start_with_bind_address(address)?,
            can_accept_clients: true,
            session_id: None,
            metadata: ServerMetadata::default(),
        })
    }
}

impl SingleServer {
    /// Sets the metadata the server will use
    pub fn set_metadata(&mut self, metadata: ServerMetadata) {
        self.metadata = metadata;
    }

    /// Set the first connected client as a host
    /// Will inform all clients that a new host has been designated, and the session_id will be sent to the host
    fn set_next_host(&mut self) -> Result<()> {
        let target_addrs = self.clients.all_addresses();

        // If there are other clients, make the first client the host
        let host = match self.clients.make_first_host() {
            Some(c) => c,
            None => return Ok(()),
        };

        // Let everyone know of the new host change
        self.socket
            .send_to_multiple(target_addrs, &MainPayloads::MakeHost { client_id: host.id })?;

        // Send the current session id to the host
        if let Some(session_id) = self.session_id.clone() {
            self.socket
                .send_to(host.addr, &MainPayloads::SessionDetails { session_id })?;
        }

        // Send new delgations with new host
        self.send_delegations()?;

        Ok(())
    }

    /// The current state of all clients and control delegations will be sent to the incoming client
    /// Other clients will receive the new client's state
    fn send_new_connected_client_info(&self, incoming_client: &ClientInfo) -> Result<()> {
        // Send other connected client info to incoming client
        for other_client in self.clients.all() {
            if other_client.id == incoming_client.id {
                continue;
            }
            self.socket
                .send_to(incoming_client.addr, &other_client.into_connected_payload())?
        }

        // Send current control delegations to the incoming client
        self.socket.send_to(
            incoming_client.addr,
            &MainPayloads::ControlDelegations {
                delegations: self.clients.delegations().clone(),
            },
        )?;

        // Send incoming client to other connected clients
        self.socket.send_to_multiple(
            self.clients.all_addresses_except(&incoming_client.addr),
            &incoming_client.into_connected_payload(),
        )?;

        Ok(())
    }

    /// Whether the provided session_id matches the session_id the server has
    fn is_valid_session_id(&self, session_id: &str) -> bool {
        self.session_id
            .as_ref()
            .map(|x| x == session_id)
            .unwrap_or(true)
    }

    /// Whether the provided version matches the server's version
    fn is_valid_version(&self, version: &str) -> bool {
        self.metadata
            .version
            .as_ref()
            .map(|server_version| version == server_version)
            .unwrap_or(true)
    }

    /// Sends the current client delegations to all clients
    fn send_delegations(&self) -> Result<()> {
        Ok(self.socket.send_to_multiple(
            self.clients.all_addresses(), // Control delegations will be sent to this client later
            &MainPayloads::ControlDelegations {
                delegations: self.clients.delegations().clone(),
            },
        )?)
    }

    /// Processes payload messages coming from addr
    pub fn process_payload(&mut self, payload: MainPayloads, addr: SocketAddr) -> Result<()> {
        let mut should_relay = false;

        match &payload {
            // Used
            MainPayloads::Hello {
                session_id,
                version,
            } => {
                self.on_hello_received(session_id, version, addr)?;
            }
            MainPayloads::Name { name } => {
                self.on_name_received(name, addr)?;
            }
            MainPayloads::SessionDetails { session_id } => {
                self.on_session_received(session_id)?;
            }
            MainPayloads::Update { changed, time, .. } => {
                should_relay = true;
            }
            MainPayloads::TransferControl { delegation, to } => {
                self.on_transfer_control_received(delegation, to);
            }
            _ => {}
        }

        if should_relay {
            self.socket
                .send_to_multiple(self.clients.all_addresses_except(&addr), &payload)?;
        }

        Ok(())
    }

    fn on_hello_received(
        &mut self,
        session_id: &String,
        version: &String,
        addr: SocketAddr,
    ) -> Result<()> {
        // Multiple hello payloads might've been sent
        if self.clients.get_id_for(addr).is_some() {
            return Ok(());
        }

        if !self.is_valid_session_id(session_id) {
            self.socket.send_to(addr, &MainPayloads::InvalidSession)?;
            return Ok(());
        }

        if !self.is_valid_version(version) {
            self.socket.send_to(
                addr,
                &MainPayloads::InvalidVersion {
                    server_version: self.metadata.version.clone().unwrap_or_default(),
                },
            )?;
        }

        self.socket.send_to(
            addr,
            &MainPayloads::Hello {
                session_id: self.session_id.clone().unwrap_or_default(),
                version: self.metadata.version.clone().unwrap_or_default(),
            },
        )?;

        // Passes all checks, register client
        self.clients.add(addr, None);

        Ok(())
    }

    /// Stores the client's name, assigns an id to the client and sends it back, handles setting a new host if applicaable
    /// and then relays client states between the incoming client and connected clients
    fn on_name_received(&mut self, name: &String, addr: SocketAddr) -> Result<()> {
        let id = match self.clients.get_id_for(addr) {
            Some(id) => id,
            None => return Ok(()),
        };

        self.clients.set_name(&id, name.clone());

        // Send id to incoming client
        self.socket.send_to(
            addr,
            &MainPayloads::Welcome {
                client_id: id,
                name: name.clone(),
            },
        )?;

        if self.clients.get_host().is_none() {
            self.set_next_host()?;
        }

        self.send_new_connected_client_info(self.clients.get(&id).unwrap())
    }

    /// Stores the session_id obtained from the rendezvous server, and allows new clients to be accepted
    fn on_session_received(&mut self, session_id: &String) -> Result<()> {
        self.session_id = Some(session_id.clone());
        self.can_accept_clients = true;

        Ok(())
    }

    /// Checks that the client has the delegation they're trying to transfer, then update the client's state and
    /// inform everyone else of the change
    fn on_transfer_control_received(
        &mut self,
        delegation: &ControlSurfaces,
        to: &ClientId,
    ) -> Result<()> {
        let has_delegation = self
            .clients
            .get_current_delegated_client(delegation)
            .map_or(false, |current_delegatee| current_delegatee == to);

        if !has_delegation {
            // TODO: throw error or something
            return Ok(());
        }

        self.clients.delegate(delegation.clone(), *to);
        self.send_delegations()
    }

    /// Request a session_id from the rendezvous server. Bails if no rendezvous server is set
    pub fn request_session(&mut self) -> Result<()> {
        let rendezvous_server = match self.metadata.rendezvous_server {
            Some(a) => a,
            None => bail!("A session was requested but no rendezvous server was set!"),
        };

        self.socket.send_to(
            rendezvous_server,
            &MainPayloads::RequestSession { self_hosted: true },
        )?;

        self.can_accept_clients = false;

        Ok(())
    }

    /// Gets a reference to the session_id the server received if it has
    pub fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    /// Processes socket messages
    fn process_messages(
        &mut self,
        messages: impl IntoIterator<Item = Message<MainPayloads>>,
    ) -> Result<()> {
        for message in messages {
            match message {
                Message::NewConnection(_) => {}
                Message::LostConnection(addr) => {
                    self.on_client_connection_lost(addr)?;
                }
                Message::Payload(payload, addr) => {
                    self.on_payload_received(payload, addr)?;
                }
            }
        }

        Ok(())
    }

    /// Established connection was lost... first, try to find another host if applicable, then inform others
    /// of the disconnect
    fn on_client_connection_lost(&mut self, addr: SocketAddr) -> Result<()> {
        let client = match self.clients.remove_from_addr(&addr) {
            Some(c) => c,
            None => return Ok(()),
        };

        self.set_next_host()?;

        self.socket.send_to_multiple(
            self.clients.all_addresses(),
            &MainPayloads::ClientRemoved {
                client_id: client.id,
            },
        )?;

        Ok(())
    }

    fn on_payload_received(&mut self, payload: MainPayloads, addr: SocketAddr) -> Result<()> {
        // Server still starting up
        let is_rendezvous_server = self
            .metadata
            .rendezvous_server
            .map_or(false, |rendezvous_address| rendezvous_address == addr);
        // Do not allow any messages other than the rendezvous server
        if !is_rendezvous_server && !self.can_accept_clients {
            return Ok(());
        }

        self.process_payload(payload, addr)
    }

    /// Gets the address from the underlying socket
    pub fn get_address(&self) -> SocketAddr {
        self.socket.get_address()
    }

    /// Polls and processes
    pub fn poll(&mut self) -> Result<()> {
        // Receive messages
        let messages = self.socket.poll::<MainPayloads>();

        self.process_messages(messages.into_iter())?;

        Ok(())
    }
}

#[derive(Default)]
/// Metadata for handshaking with clients and the rendezvous server
pub struct ServerMetadata {
    version: Option<String>,
    rendezvous_server: Option<SocketAddr>,
}

impl ServerMetadata {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_rendezvous_server(mut self, rendezvous_server: SocketAddr) -> Self {
        self.rendezvous_server = Some(rendezvous_server);
        self
    }
}
