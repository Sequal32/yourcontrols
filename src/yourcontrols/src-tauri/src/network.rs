use anyhow::{bail, Result};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
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
}

impl Network {
    pub fn new() -> Self {
        Self {
            rendezvous_socket: BaseSocket::start_with_port(0).expect("socket can't start"),
            direct_socket: None,
            server: None,
            handshake: None,
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

    pub fn request_session(&mut self) -> Result<()> {
        self.rendezvous_socket.send_to(
            RENDEZVOUS_SERVER.parse().expect("bad server address"),
            &MainPayloads::RequestSession { self_hosted: true },
        )?;

        Ok(())
    }

    pub fn start_direct(&mut self, port: u16) -> Result<()> {
        let server = SingleServer::start_with_port(port)?;

        self.connect_to_address(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            port,
        )))?;

        self.server = Some(server);

        Ok(())
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
