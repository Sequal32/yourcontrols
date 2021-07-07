use anyhow::{bail, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use yourcontrols_hoster::SingleServer;
use yourcontrols_net::{
    BaseSocket, DirectHandshake, Handshake, HandshakeConfig, MainPayloads, Message,
};
use yourcontrols_types::{ChangedDatum, Time};

const RENDEZVOUS_SERVER: &str = "127.0.0.1:25070";

pub struct Network {
    rendezvous_socket: BaseSocket,
    direct_socket: Option<BaseSocket>,
    server: Option<SingleServer>,
    handshake: Option<Box<dyn Handshake>>,
    session_id: Option<String>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            rendezvous_socket: BaseSocket::start_with_port(0).expect("socket can't start"),
            direct_socket: None,
            server: None,
            handshake: None,
            session_id: None,
        }
    }

    pub fn process_payload(
        &mut self,
        payload: MainPayloads,
        addr: SocketAddr,
    ) -> Result<Option<NetworkEvent>> {
        match payload {
            MainPayloads::Hello {
                session_id,
                version,
            } => {}
            MainPayloads::RequestSession { self_hosted } => {}
            MainPayloads::SessionDetails { session_id } => {
                return Ok(Some(NetworkEvent::SessionReceived { session_id }))
            }
            MainPayloads::AttemptConnection {
                public_ip,
                local_ip,
            } => {}
            MainPayloads::InvalidSession => {}
            MainPayloads::InvalidVersion { server_version } => {}
            MainPayloads::Update { changed, time, .. } => {
                return Ok(Some(NetworkEvent::Update { changed, time }))
            }
            _ => {}
        }

        Ok(None)
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
        self.server = Some(SingleServer::start_with_port(port)?);
        self.connect_to_server()?;

        Ok(())
    }

    pub fn start_cloud_p2p(&mut self) -> Result<()> {
        let mut server = SingleServer::start()?;
        server.set_rendezvous_server("127.0.0.1:25070".parse().unwrap()); // TODO: temp rendezvous server
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

    fn send_direct_message(&mut self, payload: MainPayloads, to: SocketAddr) -> Result<()> {
        if let Some(direct_socket) = self.direct_socket.as_mut() {
            direct_socket.send_to(to, &payload)?;
        }
        Ok(())
    }

    fn send_direct_message_multi(
        &mut self,
        payload: MainPayloads,
        to: Vec<SocketAddr>,
    ) -> Result<()> {
        if let Some(direct_socket) = self.direct_socket.as_mut() {
            direct_socket.send_to_multiple(to, &payload)?;
        }
        Ok(())
    }

    fn handle_handshake(&mut self) -> Result<Option<NetworkEvent>> {
        let handshake = match self.handshake.take() {
            Some(h) => h,
            None => return Ok(None),
        };

        match handshake.handshake() {
            Ok(handshake) => {
                self.direct_socket = Some(handshake.inner());
                return Ok(Some(NetworkEvent::Connected));
            }
            Err(e) => match e {
                yourcontrols_net::HandshakeFail::InProgress(handshake) => {
                    self.handshake = Some(handshake);
                }
                _ => bail!("{:?}", e), // TODO: better error
            },
        }

        Ok(None)
    }

    pub fn send_update(
        &mut self,
        time: Time,
        unreliable: Vec<ChangedDatum>,
        reliable: Vec<ChangedDatum>,
        to: Vec<SocketAddr>,
    ) -> Result<()> {
        self.send_direct_message_multi(
            MainPayloads::Update {
                is_reliable: false,
                time,
                changed: unreliable,
            },
            to.clone(),
        )?;

        self.send_direct_message_multi(
            MainPayloads::Update {
                is_reliable: true,
                time,
                changed: reliable,
            },
            to,
        )?;

        Ok(())
    }

    pub fn step(&mut self) -> Result<Vec<NetworkEvent>> {
        let mut events = Vec::new();

        let mut messages = self.rendezvous_socket.poll();

        if let Some(direct_socket) = self.direct_socket.as_mut() {
            messages.extend(direct_socket.poll().into_iter());
        }

        self.process_messages(messages, &mut events)?;

        if let Some(event) = self.handle_handshake()? {
            events.push(event);
        }

        if self.session_id.is_none() {
            if let Some(session_id) = self.server.as_ref().and_then(SingleServer::session_id) {
                // Tell program we've received session
                events.push(NetworkEvent::SessionReceived {
                    session_id: session_id.clone(),
                });
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
                self.session_id = Some(session_id.clone())
            }
        }

        if let Some(server) = self.server.as_mut() {
            server.poll()?;
        }

        Ok(events)
    }
}

#[derive(Debug)]
pub enum NetworkEvent {
    SessionReceived {
        session_id: String,
    },
    Connected,
    Update {
        changed: Vec<ChangedDatum>,
        time: Time,
    },
}
