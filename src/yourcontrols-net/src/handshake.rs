use std::{net::SocketAddr, time::Instant};
use yourcontrols_types::Error;

use crate::payloads::MainPayloads;

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
                &MainPayloads::Hello {
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
        for message in self.socket.poll::<MainPayloads>() {
            match message {
                Message::Payload(MainPayloads::Hello { .. }, _) => return Ok(self),
                Message::Payload(MainPayloads::InvalidVersion, _) => {
                    return Err(HandshakeFail::InvalidVersion)
                }
                _ => {}
            }
        }

        Err(HandshakeFail::InProgress(self))
    }

    fn get_config(&self) -> &HandshakeConfig {
        &self.config
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
                &MainPayloads::Hello {
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
        for message in self.socket.poll::<MainPayloads>() {
            match message {
                Message::Payload(
                    MainPayloads::AttemptConnection {
                        public_ip,
                        local_ip,
                    },
                    _,
                ) => {
                    let (public_ip, local_ip) = Self::get_ips(public_ip, local_ip)
                        .map_err(|_| HandshakeFail::InvalidData)?;

                    return Err(HandshakeFail::InProgress(Box::new(DirectHandshake::new(
                        self.socket,
                        self.config,
                        public_ip,
                        Some(local_ip),
                    ))));
                }
                _ => {}
            }
        }

        Err(HandshakeFail::InProgress(self))
    }

    fn get_config(&self) -> &HandshakeConfig {
        &self.config
    }
}

pub struct SessionHostHandshake {
    socket: BaseSocket,
    config: HandshakeConfig,
    retry: RetryHelper,
}

impl SessionHostHandshake {
    pub fn new(socket: BaseSocket, config: HandshakeConfig) -> Self {
        Self {
            socket: socket,
            retry: RetryHelper::new(config.retry_count),
            config,
        }
    }

    fn request_hosting(&self) {
        self.socket
            .send_to(
                self.config.rendezvous_address,
                &MainPayloads::RequestSession {
                    self_hosted: self.config.self_hosted,
                },
            )
            .unwrap();
    }
}

impl Handshake for SessionHostHandshake {
    fn handshake(mut self: Box<Self>) -> Result<Box<dyn Handshake>, HandshakeFail> {
        if self.retry.done() {
            return Err(HandshakeFail::NoMessage);
        }

        // Request IP
        if self.retry.should_retry() {
            self.request_hosting()
        }

        // Receive two ips to attempt connection to using a DirectHandshake
        for message in self.socket.poll::<MainPayloads>() {
            match message {
                Message::Payload(MainPayloads::SessionDetails { session_id }, _) => {
                    self.config.session_id = session_id;
                    return Ok(self);
                }
                _ => {}
            }
        }

        Err(HandshakeFail::InProgress(self))
    }

    fn get_config(&self) -> &HandshakeConfig {
        &self.config
    }

    fn inner(self) -> BaseSocket {
        self.socket
    }
}

pub trait Handshake {
    fn inner(self) -> BaseSocket;
    fn handshake(self: Box<Self>) -> Result<Box<dyn Handshake>, HandshakeFail>;
    fn get_config(&self) -> &HandshakeConfig;
    fn get_session_id(&self) -> &String {
        &self.get_config().session_id
    }
}

impl std::fmt::Debug for dyn Handshake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get_config())
    }
}

#[derive(Debug)]
pub struct HandshakeConfig {
    pub rendezvous_address: SocketAddr,
    pub retry_count: u16,
    pub version: String,
    pub session_id: String,
    pub self_hosted: bool,
}

impl Default for HandshakeConfig {
    fn default() -> Self {
        Self {
            rendezvous_address: "127.0.0.1:0".parse().unwrap(),
            retry_count: 5,
            version: String::new(),
            session_id: String::new(),
            self_hosted: false,
        }
    }
}

#[derive(Debug)]
pub enum HandshakeFail {
    NoMessage,
    InvalidData,
    InvalidVersion,
    InProgress(Box<dyn Handshake>),
}
