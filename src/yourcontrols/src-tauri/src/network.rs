use anyhow::Result;
use std::net::SocketAddr;
use yourcontrols_net::{BaseSocket, MainPayloads, Message};
use yourcontrols_types::{ChangedDatum, Time};

const RENDEZVOUS_SERVER: &str = "127.0.0.1:25070";

pub struct Network {
    rendezvous_socket: BaseSocket,
    direct_socket: Option<BaseSocket>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            rendezvous_socket: BaseSocket::start(0).expect("socket can't start"),
            direct_socket: None,
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
        self.direct_socket = Some(BaseSocket::start(port)?);
        Ok(())
    }

    fn send_direct_message(&mut self, payload: MainPayloads, to: SocketAddr) {
        if let Some(direct_socket) = self.direct_socket.as_mut() {
            direct_socket.send_to(to, &payload);
        }
    }

    fn send_direct_message_multi(&mut self, payload: MainPayloads, to: Vec<SocketAddr>) {
        if let Some(direct_socket) = self.direct_socket.as_mut() {
            direct_socket.send_to_multiple(to, &payload);
        }
    }

    pub fn send_update(
        &mut self,
        time: Time,
        unreliable: Vec<ChangedDatum>,
        reliable: Vec<ChangedDatum>,
        to: Vec<SocketAddr>,
    ) {
        self.send_direct_message_multi(
            MainPayloads::Update {
                is_reliable: false,
                time,
                changed: unreliable,
            },
            to.clone(),
        );

        self.send_direct_message_multi(
            MainPayloads::Update {
                is_reliable: true,
                time,
                changed: reliable,
            },
            to,
        );
    }

    pub fn step(&mut self) -> Result<Vec<NetworkEvent>> {
        let mut events = Vec::new();

        let mut messages = self.rendezvous_socket.poll();

        if let Some(direct_socket) = self.direct_socket.as_mut() {
            messages.extend(direct_socket.poll().into_iter());
        }

        self.process_messages(messages, &mut events)?;

        Ok(events)
    }
}

#[derive(Debug)]
pub enum NetworkEvent {
    SessionReceived {
        session_id: String,
    },
    Update {
        changed: Vec<ChangedDatum>,
        time: Time,
    },
}