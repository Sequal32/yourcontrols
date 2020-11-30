use crossbeam_channel::{Receiver, Sender};
use dns_lookup::lookup_host;
use laminar::{Packet, SocketEvent};
use std::{fmt::Display, net::IpAddr, net::SocketAddr, net::SocketAddrV4, net::SocketAddrV6, time::Duration};
use std::time::SystemTime;

use crate::{definitions::AllNeedSync, util::HostnameLookupError};

use super::Payloads;

pub const MAX_PUNCH_RETRIES: u8 = 5;
pub const LOOP_SLEEP_TIME_MS: u64 = 10;

const HEARTBEAT_INTERVAL_MS: u64 = 1000;
const RENDEZVOUS_SERVER_HOSTNAME: &str = "cloudserver.yourcontrols.xyz";
const RENDEZVOUS_PORT: u16 = 5555;

pub fn get_bind_address(is_ipv6: bool, port: Option<u16>) -> SocketAddr {
    let bind_string = format!("{}:{}", if is_ipv6 {"::"} else {"0.0.0.0"}, port.unwrap_or(0));
    bind_string.parse().unwrap()
}

pub fn match_ip_address_to_socket_addr(ip: IpAddr, port: u16) -> SocketAddr {
    match ip {
        IpAddr::V4(ip) => return SocketAddr::V4(
            SocketAddrV4::new(ip, port)
        ),
        IpAddr::V6(ip) => return SocketAddr::V6(
            SocketAddrV6::new(ip, port, 0, 0)
        )
    }
}

pub fn get_rendezvous_server(is_ipv6: bool) -> Result<SocketAddr, HostnameLookupError> {
    for ip in lookup_host(RENDEZVOUS_SERVER_HOSTNAME)? {
        if (ip.is_ipv6() && !is_ipv6) || (ip.is_ipv4() && is_ipv6) {continue;}
        return Ok(match_ip_address_to_socket_addr(ip, RENDEZVOUS_PORT))
    }
    Err(HostnameLookupError::WrongIpVersion)
}

pub fn get_socket_config(timeout: u64) -> laminar::Config {
    laminar::Config {
        heartbeat_interval: Some(Duration::from_millis(HEARTBEAT_INTERVAL_MS)),
        idle_connection_timeout: Duration::from_secs(timeout),
        receive_buffer_max_size: 65536,
        ..Default::default()
    }
}

#[derive(Debug)]
pub enum Event {
    ConnectionEstablished,
    UnablePunchthrough,
    SessionIdFetchFailed,
    ConnectionLost(String)
}

#[derive(Debug)]
pub enum ReceiveMessage {
    Payload(Payloads),
    Event(Event)
}

// Errors
#[derive(Debug)]
pub enum StartClientError {
    FetchRendezvousError(HostnameLookupError),
    SocketError(laminar::ErrorKind),
    PortForwardError(PortForwardResult)
}

impl Display for StartClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartClientError::FetchRendezvousError(e) => write!(f, "{}", e),
            StartClientError::SocketError(e) => write!(f, "Could not initialize socket! Reason: {}", e),
            StartClientError::PortForwardError(e) => write!(f, "Could not automatically port forward! Reason: {}", e)
        }
    }
}

impl From<HostnameLookupError> for StartClientError {
    fn from(e: HostnameLookupError) -> Self {
        StartClientError::FetchRendezvousError(e)
    }
}

impl From<PortForwardResult> for StartClientError {
    fn from(e: PortForwardResult) -> Self {
        StartClientError::PortForwardError(e)
    }
}

impl From<laminar::ErrorKind> for StartClientError {
    fn from(e: laminar::ErrorKind) -> Self {
        StartClientError::SocketError(e)
    }
}


#[derive(Debug)]
pub enum PortForwardResult {
    GatewayNotFound(igd::SearchError),
    LocalAddrNotFound,
    LocalAddrNotIPv4(String),
    AddPortError(igd::AddPortError)
}

impl Display for PortForwardResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortForwardResult::GatewayNotFound(e) => write!(f, "Gateway not found: {}", e),
            PortForwardResult::LocalAddrNotFound => write!(f, "Could not get local address."),
            PortForwardResult::AddPortError(e) => write!(f, "Could not add port: {}", e),
            PortForwardResult::LocalAddrNotIPv4(parse_string) => write!(f, "{} is not IPv4", parse_string)
        }
    }
}

pub trait TransferClient {
    fn get_connected_count(&self) -> u16;
    fn is_server(&self) -> bool;

    fn get_transmitter(&self) -> &Sender<Payloads>;
    fn get_server_transmitter(&self) -> &Sender<ReceiveMessage>;
    fn get_receiver(&self) -> &Receiver<ReceiveMessage>;
    fn get_server_name(&self) -> &str;
    fn get_session_id(&self) -> Option<String>;
    // Application specific functions
    fn stop(&mut self, reason: String);

    fn update(&self, data: AllNeedSync, is_unreliable: bool) {
        self.get_transmitter().try_send(Payloads::Update {
            data,
            from: self.get_server_name().to_string(),
            is_unreliable
        }).ok();
    }

    fn get_next_message(&self) -> Result<ReceiveMessage, crossbeam_channel::TryRecvError> {
        return self.get_receiver().try_recv();
    }

    fn transfer_control(&self, target: String) {
        let message = Payloads::TransferControl {
            from: self.get_server_name().to_string(),
            to: target
        };
        self.get_transmitter().try_send(message.clone()).ok();
        self.get_server_transmitter().try_send(ReceiveMessage::Payload(message)).ok();
    }

    fn take_control(&self, from: String) {
        let message = Payloads::TransferControl {
            from,
            to: self.get_server_name().to_string(),
        };

        self.get_transmitter().try_send(message.clone()).ok();
        self.get_server_transmitter().try_send(ReceiveMessage::Payload(message)).ok();
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        self.get_transmitter().try_send(Payloads::SetObserver {
            from: self.get_server_name().to_string(),
            to: target,
            is_observer: is_observer
        }).ok();
    }

    fn send_init(&self, version: String) {
        self.get_transmitter().try_send(Payloads::InitHandshake {
            name: self.get_server_name().to_string(),
            version
        }).ok();
    }
}

pub struct SenderReceiver {
    sender: Sender<Packet>,
    receiver: Receiver<SocketEvent>
}

impl SenderReceiver {
    pub fn new(sender: Sender<Packet>, receiver: Receiver<SocketEvent>) -> Self {
        Self { sender, receiver }
    }

    pub fn get_sender(&mut self) -> &mut Sender<Packet> {
        &mut self.sender
    }
    
    pub fn get_receiver(&mut self) -> &mut Receiver<SocketEvent> {
        &mut self.receiver
    }
}