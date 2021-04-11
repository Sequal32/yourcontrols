use std::{net::SocketAddr, time::Instant};

use laminar::Packet;
use serde::{Deserialize, Serialize};
use yourcontrols_types::Error;

use super::base::{BaseSocket, Message, Payload};

struct RetryHelper {
    current_retry: u16,
    max_retries: u16,
    last_retry: Option<Instant>,
}

impl RetryHelper {
    pub fn new(max_retries: u16) -> Self {
        Self {
            current_retry: 0,
            max_retries: max_retries,
            last_retry: None,
        }
    }

    pub fn should_retry(&mut self) -> bool {
        let should_retry = self
            .last_retry
            .map(|x| x.elapsed().as_secs() >= 2)
            .unwrap_or(true);

        if should_retry {
            self.current_retry += 1;
            self.last_retry = Some(Instant::now())
        };

        should_retry
    }

    pub fn done(&self) -> bool {
        self.current_retry > self.max_retries
    }

    pub fn reset(&mut self) {
        self.current_retry = 0;
        self.last_retry = None;
    }
}

pub struct DirectHandshake {
    socket: BaseSocket,
    config: HandshakeConfig,
    target_address_1: SocketAddr,
    target_address_2: Option<SocketAddr>,
    retry: RetryHelper,
}

impl DirectHandshake {
    pub fn new(
        socket: BaseSocket,
        config: HandshakeConfig,
        target_address_1: SocketAddr,
        target_address_2: Option<SocketAddr>,
    ) -> Self {
        Self {
            socket: socket,
            retry: RetryHelper::new(config.retry_count),
            target_address_1,
            target_address_2,
            config,
        }
    }

    fn send_hello(socket: &mut BaseSocket, addr: SocketAddr, config: &HandshakeConfig) {
        socket
            .send_to(
                addr,
                &HandshakePayloads::Hello {
                    session_id: config.session_id.clone(),
                    version: config.version.clone(),
                },
            )
            .ok();
    }
}

impl Handshake for DirectHandshake {
    fn inner(self) -> BaseSocket {
        self.socket
    }

    fn handshake(mut self: Box<Self>) -> Result<Box<dyn Handshake>, HandshakeFail> {
        if self.retry.done() {
            return Err(HandshakeFail::NoMessage);
        }

        // Attempt handshake
        if self.retry.should_retry() {
            Self::send_hello(&mut self.socket, self.target_address_1, &self.config);

            if let Some(target_address_2) = self.target_address_2.as_ref().cloned() {
                Self::send_hello(&mut self.socket, target_address_2, &self.config);
            }
        }

        // Receive a handshake OK
        for message in self.socket.poll::<HandshakePayloads>() {
            match message {
                Message::Payload(HandshakePayloads::Hello { .. }) => return Ok(self),
                Message::Payload(HandshakePayloads::InvalidVersion) => {
                    return Err(HandshakeFail::InvalidVersion)
                }
                _ => {}
            }
        }

        Err(HandshakeFail::InProgress(self))
    }
}

pub struct PunchthroughHandshake {
    socket: BaseSocket,
    config: HandshakeConfig,
    retry: RetryHelper,
}

impl PunchthroughHandshake {
    pub fn new(socket: BaseSocket, config: HandshakeConfig) -> Self {
        Self {
            socket: socket,
            retry: RetryHelper::new(config.retry_count),
            config,
        }
    }

    fn get_ips(public_ip: String, local_ip: String) -> Result<(SocketAddr, SocketAddr), Error> {
        Ok((
            String::from_utf8(base64::decode(public_ip)?)?.parse()?,
            String::from_utf8(base64::decode(local_ip)?)?.parse()?,
        ))
    }

    fn request_connection_details(&self) {
        self.socket
            .send_to(
                self.config.rendezvous_address,
                &HandshakePayloads::Hello {
                    session_id: self.config.session_id.clone(),
                    version: self.config.version.clone(),
                },
            )
            .ok();
    }
}

impl Handshake for PunchthroughHandshake {
    fn inner(self) -> BaseSocket {
        self.socket
    }

    fn handshake(mut self: Box<Self>) -> Result<Box<dyn Handshake>, HandshakeFail> {
        if self.retry.done() {
            return Err(HandshakeFail::NoMessage);
        }

        // Request IP
        if self.retry.should_retry() {
            self.request_connection_details()
        }

        // Receive two ips to attempt connection to using a DirectHandshake
        for message in self.socket.poll::<HandshakePayloads>() {
            match message {
                Message::Payload(HandshakePayloads::AttemptConnection {
                    public_ip,
                    local_ip,
                }) => {
                    let (public_ip, local_ip) = Self::get_ips(public_ip, local_ip)
                        .map_err(|_| HandshakeFail::InvalidData)?;

                    return Ok(Box::new(DirectHandshake::new(
                        self.socket,
                        self.config,
                        public_ip,
                        Some(local_ip),
                    )));
                }
                _ => {}
            }
        }

        Err(HandshakeFail::InProgress(self))
    }
}

pub trait Handshake {
    fn inner(self) -> BaseSocket;
    fn handshake(self: Box<Self>) -> Result<Box<dyn Handshake>, HandshakeFail>;
}

pub struct SessionHostHandshake {}

pub struct HandshakeConfig {
    rendezvous_address: SocketAddr,
    retry_count: u16,
    version: String,
    session_id: String,
}

impl Default for HandshakeConfig {
    fn default() -> Self {
        Self {
            rendezvous_address: "127.0.0.1:0".parse().unwrap(),
            retry_count: 5,
            version: String::new(),
            session_id: String::new(),
        }
    }
}

pub enum HandshakeFail {
    Completed,
    RendezvousFail,
    NoMessage,
    InvalidData,
    InvalidVersion,
    InProgress(Box<dyn Handshake>),
}

#[derive(Serialize, Deserialize)]
pub enum HandshakePayloads {
    Hello { session_id: String, version: String },
    AttemptConnection { public_ip: String, local_ip: String }, // Base64 encoded strings
    InvalidSession,
    InvalidVersion,
}

impl Payload for HandshakePayloads {
    fn get_packet(&self, addr: SocketAddr, bytes: Vec<u8>) -> laminar::Packet {
        Packet::reliable_ordered(addr, bytes, None)
    }
}
