mod clients;
#[cfg(test)]
mod test;

use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::Result;
use clients::Clients;
use yourcontrols_net::{BaseSocket, MainPayloads, Message};

use self::clients::ClientInfo;

pub struct SingleServer {
    clients: Clients,
    socket: BaseSocket,
    can_accept_clients: bool,
    session_id: Option<String>,
    version: Option<String>,
    rendezvous_server: Option<SocketAddr>,
}

impl SingleServer {
    pub fn start() -> Result<Self> {
        Self::start_with_bind_address("0.0.0.0:0")
    }

    pub fn start_with_port(port: u16) -> Result<Self> {
        Self::start_with_bind_address(format!("0.0.0.0:{}", port))
    }

    pub fn start_with_bind_address(address: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self {
            clients: Clients::new(),
            socket: BaseSocket::start_with_bind_address(address)?,
            can_accept_clients: true,
            session_id: None,
            version: None,
            rendezvous_server: None,
        })
    }

    pub fn set_version(&mut self, version: String) {
        self.version = Some(version);
    }

    pub fn set_rendezvous_server(&mut self, addr: SocketAddr) {
        self.rendezvous_server = Some(addr);
    }

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

        Ok(())
    }

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

    fn is_valid_session_id(&self, session_id: &str) -> bool {
        self.session_id
            .as_ref()
            .map(|x| x == session_id)
            .unwrap_or(true)
    }

    fn is_valid_version(&self, version: &str) -> bool {
        self.version
            .as_ref()
            .map(|server_version| version == server_version)
            .unwrap_or(true)
    }

    fn send_delegations(&self, except_to: &SocketAddr) -> Result<()> {
        Ok(self.socket.send_to_multiple(
            self.clients.all_addresses_except(&except_to), // Control delegations will be sent to this client later
            &MainPayloads::ControlDelegations {
                delegations: self.clients.delegations().clone(),
            },
        )?)
    }

    pub fn process_payload(&mut self, payload: MainPayloads, addr: SocketAddr) -> Result<()> {
        let mut should_relay = false;

        match &payload {
            // Used
            MainPayloads::Hello {
                session_id,
                version,
            } => {
                if !self.is_valid_session_id(session_id) {
                    self.socket.send_to(addr, &MainPayloads::InvalidSession)?;
                    return Ok(());
                }

                if !self.is_valid_version(version) {
                    self.socket.send_to(
                        addr,
                        &MainPayloads::InvalidVersion {
                            server_version: self.version.clone().unwrap_or_default(),
                        },
                    )?;
                }

                self.socket.send_to(
                    addr,
                    &MainPayloads::Hello {
                        session_id: self.version.clone().unwrap_or_default(),
                        version: self.version.clone().unwrap_or_default(),
                    },
                )?;

                // Passes all checks, register client
                self.clients.add(addr, None);
            }
            MainPayloads::Name { name } => {
                let id = match self.clients.get_id_for(addr) {
                    Some(id) => id,
                    None => return Ok(()),
                };

                self.clients.set_name(&id, name.clone());

                // Send id to incoming client
                self.socket
                    .send_to(addr, &MainPayloads::Welcome { client_id: id })?;

                if self.clients.get_host().is_none() {
                    self.set_next_host()?;
                    // Send out new delegations as they might have changed upon the host switch
                    self.send_delegations(&addr)?;
                }

                self.send_new_connected_client_info(self.clients.get(&id).unwrap())?;
            }
            MainPayloads::SessionDetails { session_id } => {
                self.session_id = Some(session_id.clone());
                self.can_accept_clients = true;
            }
            MainPayloads::Update { changed, time, .. } => {
                should_relay = true;
            }
            MainPayloads::TransferControl { delegation, to } => {
                let has_delegation = self
                    .clients
                    .get_current_delegated_client(delegation)
                    .map_or(false, |current_delegatee| current_delegatee == to);

                if !has_delegation {
                    // TODO: throw error or something
                    return Ok(());
                }

                self.clients.delegate(delegation.clone(), *to);
                self.send_delegations(&addr)?;
            }
            _ => {}
        }

        if should_relay {
            self.socket
                .send_to_multiple(self.clients.all_addresses_except(&addr), &payload)?;
        }

        Ok(())
    }

    pub fn request_session(&mut self) -> Result<()> {
        if let Some(rendezvous_server) = self.rendezvous_server {
            self.socket.send_to(
                rendezvous_server,
                &MainPayloads::RequestSession { self_hosted: true },
            )?;

            self.can_accept_clients = false;
        }

        Ok(())
    }

    pub fn session_id(&mut self) -> Option<&String> {
        self.session_id.as_ref()
    }

    fn process_messages(
        &mut self,
        messages: impl IntoIterator<Item = Message<MainPayloads>>,
    ) -> Result<()> {
        for message in messages {
            match message {
                Message::NewConnection(_) => {}
                Message::LostConnection(addr) => {
                    if let Some(client) = self.clients.remove_from_addr(&addr) {
                        self.set_next_host()?;
                        self.send_delegations(&addr)?;

                        self.socket.send_to_multiple(
                            self.clients.all_addresses(),
                            &MainPayloads::ClientRemoved { id: client.id },
                        )?;
                    }
                }
                Message::Payload(payload, addr) => {
                    // Server still starting up
                    let is_rendezvous_server = self
                        .rendezvous_server
                        .map_or(false, |rendezvous_address| rendezvous_address == addr);
                    // Do not allow any messages other than the rendezvous server
                    if !is_rendezvous_server && !self.can_accept_clients {
                        continue;
                    }

                    self.process_payload(payload, addr)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_address(&self) -> SocketAddr {
        self.socket.get_address()
    }

    pub fn poll(&mut self) -> Result<()> {
        // Receive messages
        let messages = self.socket.poll::<MainPayloads>();

        self.process_messages(messages.into_iter())?;

        Ok(())
    }
}
