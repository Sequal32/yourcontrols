use anyhow::{bail, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use yourcontrols_hoster::{ServerMetadata, SingleServer};
use yourcontrols_net::{
    BaseSocket, DirectHandshake, Handshake, HandshakeConfig, MainPayloads, Message,
    StartableNetworkObject,
};
use yourcontrols_types::{ChangedDatum, ClientId, Time};

const RENDEZVOUS_SERVER: &str = "127.0.0.1:25070";

pub struct Network {
    rendezvous_socket: BaseSocket,
    direct_socket: Option<BaseSocket>,
    server: Option<SingleServer>,
    handshake: Option<Box<dyn Handshake>>,
    session_id: Option<String>,
    connected_remote: Option<SocketAddr>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            rendezvous_socket: BaseSocket::start_with_port(0).expect("socket can't start"),
            direct_socket: None,
            server: None,
            handshake: None,
            session_id: None,
            connected_remote: None,
        }
    }

    pub fn process_payload(
        &mut self,
        payload: MainPayloads,
        addr: SocketAddr,
    ) -> Result<Option<NetworkEvent>> {
        // match &payload {
        //     MainPayloads::Hello {
        //         session_id,
        //         version,
        //     } => {}
        //     MainPayloads::RequestSession { self_hosted } => {}
        //     MainPayloads::SessionDetails { session_id } => {
        //         return Ok(Some(NetworkEvent::SessionReceived { session_id }))
        //     }
        //     MainPayloads::AttemptConnection {
        //         public_ip,
        //         local_ip,
        //     } => {}
        //     MainPayloads::InvalidSession => {}
        //     MainPayloads::InvalidVersion { server_version } => {}
        //     MainPayloads::Update {  .. } => {
        //         return Ok(Some(NetworkEvent::Payload(payload)))
        //     }
        //     MainPayloads::MakeHost { client_id } => {}
        //     MainPayloads::ClientAdded {
        //         name,
        //         id,
        //         is_host,
        //         is_observer,
        //     } => {}
        //     MainPayloads::ClientRemoved { id } => {}
        //     MainPayloads::ControlDelegations { delegations } => {}
        //     MainPayloads::Welcome { client_id } => {}
        //     _ => {}
        // }

        Ok(Some(NetworkEvent::Payload(payload)))
    }

    fn process_messages(
        &mut self,
        messages: impl IntoIterator<Item = Message<MainPayloads>>,
        events: &mut Vec<NetworkEvent>,
    ) -> Result<()> {
        for message in messages {
            match message {
                Message::NewConnection(_) => {}
                Message::LostConnection(_) => {}
                Message::Payload(payload, addr) => {
                    if let Some(event) = self.process_payload(payload, addr)? {
                        events.push(event);
                    }
                }
            }
        }

        Ok(())
    }

    // Returns a local address (127.0.0.1) with the port of the current hosted server
    fn get_local_server_addr(&self) -> Option<SocketAddr> {
        let mut addr = self.server.as_ref().unwrap().get_address();
        addr.set_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        Some(addr)
    }

    pub fn start_direct(&mut self, port: u16) -> Result<()> {
        self.server = Some(get_server(port)?);
        self.connect_to_server()?;

        Ok(())
    }

    pub fn start_cloud_p2p(&mut self) -> Result<()> {
        let mut server = get_server(0)?;
        server.request_session()?;

        self.server = Some(server);

        self.connect_to_server()
    }

    fn connect_to_server(&mut self) -> Result<()> {
        self.connect_to_address(self.get_local_server_addr().unwrap())
    }

    pub fn connect_to_address(&mut self, addr: SocketAddr) -> Result<()> {
        self.handshake = Some(Box::new(DirectHandshake::new(
            BaseSocket::start()?,
            HandshakeConfig::default(),
            addr,
            None,
        )));

        Ok(())
    }

    fn handle_handshake(&mut self, events: &mut Vec<NetworkEvent>) -> Result<()> {
        let handshake = match self.handshake.take() {
            Some(h) => h,
            None => return Ok(()),
        };

        match handshake.handshake() {
            Ok(handshake) => {
                self.connected_remote = handshake.get_remote();
                self.direct_socket = Some(handshake.inner());

                events.push(NetworkEvent::Connected);

                return Ok(());
            }
            Err(e) => match e {
                yourcontrols_net::HandshakeFail::InProgress(handshake) => {
                    self.handshake = Some(handshake);
                }
                _ => bail!("{:?}", e), // TODO: better error
            },
        }

        Ok(())
    }

    pub fn send_payload_to_server(&self, payload: MainPayloads) -> Result<()> {
        if let (Some(addr), Some(socket)) = (self.connected_remote, self.direct_socket.as_ref()) {
            socket.send_to(addr, &payload)?;
        }
        Ok(())
    }

    pub fn send_update(
        &mut self,
        client_id: ClientId,
        time: Time,
        unreliable: Vec<ChangedDatum>,
        reliable: Vec<ChangedDatum>,
    ) -> Result<()> {
        self.send_payload_to_server(MainPayloads::Update {
            client_id,
            is_reliable: false,
            time,
            changed: unreliable,
        })?;

        self.send_payload_to_server(MainPayloads::Update {
            client_id,
            is_reliable: true,
            time,
            changed: reliable,
        })?;

        Ok(())
    }

    fn check_server_received_session(&mut self, events: &mut Vec<NetworkEvent>) -> Result<()> {
        // Already received session id
        if self.session_id.is_some() {
            return Ok(());
        }

        // Attempt to get session id from the server
        let session_id = match self.server.as_ref().and_then(SingleServer::session_id) {
            Some(it) => it,
            _ => return Ok(()),
        };

        // Begin handshaking to the server
        self.handshake = Some(Box::new(DirectHandshake::new(
            BaseSocket::start()?,
            HandshakeConfig {
                // version: ,
                session_id: session_id.clone(),
                ..Default::default()
            },
            self.get_local_server_addr().unwrap(),
            None,
        )));

        self.session_id = Some(session_id.clone());

        // Tell program we've received session
        events.push(NetworkEvent::SessionReceived {
            session_id: session_id.clone(),
        });

        Ok(())
    }

    pub fn step(&mut self) -> Result<Vec<NetworkEvent>> {
        let mut events = Vec::new();

        let mut messages = self.rendezvous_socket.poll();

        if let Some(direct_socket) = self.direct_socket.as_mut() {
            messages.extend(direct_socket.poll().into_iter());
        }

        self.process_messages(messages, &mut events)?;
        self.handle_handshake(&mut events)?;
        self.check_server_received_session(&mut events)?;

        if let Some(server) = self.server.as_mut() {
            server.poll()?;
        }

        Ok(events)
    }
}

#[derive(Debug)]
pub enum NetworkEvent {
    Payload(MainPayloads),
    SessionReceived { session_id: String },
    Connected,
}

fn get_server(port: u16) -> Result<SingleServer> {
    let mut server = SingleServer::start_with_port(port)?;
    server.set_metadata(
        ServerMetadata::new().with_rendezvous_server(RENDEZVOUS_SERVER.parse().unwrap()), // TODO: temp rendezvous server
    );
    Ok(server)
}
