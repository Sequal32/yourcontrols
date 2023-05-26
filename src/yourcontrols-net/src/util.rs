use crossbeam_channel::{Receiver, Sender};
use dns_lookup::lookup_host;
use dotenv_codegen::dotenv;
use laminar::Metrics;
use socket2::{Domain, Socket, Type};
use std::net::UdpSocket;
use std::time::SystemTime;
use std::{
    net::SocketAddr,
    net::SocketAddrV4,
    net::{IpAddr, SocketAddrV6},
    time::Duration,
};
use yourcontrols_types::{AllNeedSync, Error};

use crate::messages::Payloads;

pub const MAX_PUNCH_RETRIES: u8 = 5;
pub const LOOP_SLEEP_TIME_MS: u64 = 5;
pub const HEARTBEAT_INTERVAL_MANUAL_SECS: f32 = 0.5;

const HEARTBEAT_INTERVAL_MS: u64 = 1000;
const RENDEZVOUS_SERVER_HOSTNAME: &str = dotenv!("SERVER_HOSTNAME");
const RENDEZVOUS_PORT: &str = dotenv!("SERVER_PORT");

// Types
pub type ClientSender = Sender<(Payloads, Option<String>)>;
pub type ClientReceiver = Receiver<(Payloads, Option<String>)>;
pub type ServerSender = Sender<ReceiveMessage>;
pub type ServerReceiver = Receiver<ReceiveMessage>;

pub fn get_bind_address(is_ipv6: bool, port: Option<u16>) -> SocketAddr {
    let bind_string = format!(
        "{}:{}",
        if is_ipv6 { "[::]" } else { "0.0.0.0" },
        port.unwrap_or(0)
    );
    bind_string.parse().unwrap()
}

pub fn match_ip_address_to_socket_addr(ip: IpAddr, port: u16) -> SocketAddr {
    match ip {
        IpAddr::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip, port)),
        IpAddr::V6(ip) => SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)),
    }
}

pub fn get_addr_from_hostname_and_port(
    is_ipv6: bool,
    hostname: &str,
    port: u16,
) -> Result<SocketAddr, Error> {
    for ip in lookup_host(hostname)? {
        if (ip.is_ipv6() && !is_ipv6) || (ip.is_ipv4() && is_ipv6) {
            continue;
        }
        return Ok(match_ip_address_to_socket_addr(ip, port));
    }
    Err(Error::MismatchingIpVersion)
}

pub fn get_rendezvous_server(is_ipv6: bool) -> Result<SocketAddr, Error> {
    get_addr_from_hostname_and_port(
        is_ipv6,
        RENDEZVOUS_SERVER_HOSTNAME,
        RENDEZVOUS_PORT.parse().unwrap(),
    )
}

pub fn get_socket_config(timeout: u64) -> laminar::Config {
    laminar::Config {
        heartbeat_interval: Some(Duration::from_millis(HEARTBEAT_INTERVAL_MS)),
        idle_connection_timeout: Duration::from_secs(timeout),
        receive_buffer_max_size: 65536,
        max_packets_in_flight: u16::MAX,
        max_fragments: u8::MAX, // for sending aircraft definitions
        ..Default::default()
    }
}

pub fn get_socket_duplex(port: u16) -> UdpSocket {
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, None).unwrap();
    socket.set_only_v6(false).ok();
    socket
        .bind(
            &format!("[::]:{}", port)
                .parse::<SocketAddr>()
                .unwrap()
                .into(),
        )
        .unwrap();
    socket.into()
}

pub fn get_seconds() -> f64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

pub fn get_local_endpoints_with_port(is_ipv6: bool, port: u16) -> Option<SocketAddr> {
    get_local_ip_address(is_ipv6).map(|x| SocketAddr::new(x, port))
}

pub fn get_local_ip_address(is_ipv6: bool) -> Option<IpAddr> {
    let socket = match UdpSocket::bind(if is_ipv6 { "[::]:0" } else { "0.0.0.0:0" }) {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect(if is_ipv6 {
        "[2001:4860:4860::8888]:80"
    } else {
        "8.8.8.8:80"
    }) {
        Ok(()) => (),
        Err(_) => return None,
    };

    match socket.local_addr() {
        Ok(addr) => Some(addr.ip()),
        Err(_) => None,
    }
}

pub fn is_actually_ipv4(addr: SocketAddr) -> bool {
    match addr {
        SocketAddr::V4(_) => true,
        SocketAddr::V6(v6) => matches!(v6.ip().segments(), [0, 0, 0, 0, 0, 0xFFFF, ..]),
    }
}
#[derive(Debug)]
pub enum Event {
    ConnectionEstablished,
    UnablePunchthrough,
    SessionIdFetchFailed,
    ConnectionLost(String),
    Metrics(Metrics),
}

#[derive(Debug)]
pub enum ReceiveMessage {
    Payload(Payloads),
    Event(Event),
}

pub trait TransferClient {
    fn is_host(&self) -> bool;
    fn get_transmitter(&self) -> &ClientSender;
    fn get_server_transmitter(&self) -> &ServerSender;
    fn get_receiver(&self) -> &ServerReceiver;
    fn get_server_name(&self) -> &str;
    fn get_session_id(&self) -> Option<String>;
    // Application specific functions
    fn stop(&mut self, reason: String);

    fn update(&self, data: AllNeedSync, is_unreliable: bool) {
        self.get_transmitter()
            .try_send((
                Payloads::Update {
                    data,
                    from: self.get_server_name().to_string(),
                    time: get_seconds(),
                    is_unreliable,
                },
                None,
            ))
            .ok();
    }

    fn get_next_message(&self) -> Result<ReceiveMessage, crossbeam_channel::TryRecvError> {
        return self.get_receiver().try_recv();
    }

    fn transfer_control(&self, target: String) {
        let message = Payloads::TransferControl {
            from: self.get_server_name().to_string(),
            to: target,
        };
        self.get_transmitter()
            .try_send((message.clone(), None))
            .ok();
        self.get_server_transmitter()
            .try_send(ReceiveMessage::Payload(message))
            .ok();
    }

    fn take_control(&self, from: String) {
        let message = Payloads::TransferControl {
            from,
            to: self.get_server_name().to_string(),
        };

        self.get_transmitter()
            .try_send((message.clone(), None))
            .ok();
        self.get_server_transmitter()
            .try_send(ReceiveMessage::Payload(message))
            .ok();
    }

    fn set_self_observer(&self) {
        self.get_transmitter()
            .try_send((
                Payloads::SetSelfObserver {
                    name: self.get_server_name().to_string(),
                },
                None,
            ))
            .ok();
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        self.get_transmitter()
            .try_send((
                Payloads::SetObserver {
                    from: self.get_server_name().to_string(),
                    to: target,
                    is_observer,
                },
                None,
            ))
            .ok();
    }

    fn send_ready(&self) {
        self.get_transmitter()
            .try_send((Payloads::Ready, None))
            .ok();
    }

    fn send_definitions(&self, bytes: Box<[u8]>, target: String) {
        self.get_transmitter()
            .try_send((Payloads::AircraftDefinition { bytes }, Some(target)))
            .ok();
    }
}
