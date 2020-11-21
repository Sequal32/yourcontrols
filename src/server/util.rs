use crossbeam_channel::{Receiver, Sender};
use dns_lookup::lookup_host;
use std::{net::SocketAddr, net::SocketAddrV4, net::SocketAddrV6, net::IpAddr, time::Duration};
use std::time::SystemTime;

use crate::definitions::AllNeedSync;

use super::Payloads;

pub const MAX_PUNCH_RETRIES: u8 = 5;
const HEARTBEAT_INTERVAL: u64 = 2;

const RENDEZVOUS_SERVER_HOSTNAME: &str = "holepunch.yourcontrols.xyz";

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

pub fn get_rendezvous_server(is_ipv6: bool) -> Result<SocketAddr, ()> {
    for ip in lookup_host(RENDEZVOUS_SERVER_HOSTNAME).unwrap() {
        if (ip.is_ipv6() && !is_ipv6) || (ip.is_ipv4() && is_ipv6) {continue;}
        return Ok(match_ip_address_to_socket_addr(ip, 5555))
    }
    Err(())
}

pub fn get_socket_config() -> laminar::Config {
    laminar::Config {
        heartbeat_interval: Some(Duration::from_secs(HEARTBEAT_INTERVAL)),
        idle_connection_timeout: Duration::from_secs(5),
        ..Default::default()
    }
}

fn get_seconds() -> f64 {
    return SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64()
}

pub enum Event {
    ConnectionEstablished,
    UnablePunchthrough,
    SessionIdFetchFailed,
    ConnectionLost(&'static str)
}

pub enum ReceiveMessage {
    Payload(Payloads),
    Event(Event)
}

pub trait TransferClient {
    fn get_connected_count(&self) -> u16;
    fn is_server(&self) -> bool;

    fn get_transmitter(&self) -> &Sender<Payloads>;
    fn get_receiver(&self) -> &Receiver<ReceiveMessage>;
    fn get_server_name(&self) -> &str;
    fn get_session_id(&self) -> Option<String>;
    // Application specific functions
    fn stop(&mut self, reason: &'static str);

    fn update(&self, data: AllNeedSync) {
        self.get_transmitter().send(Payloads::Update {
            data,
            time: get_seconds(),
            from: self.get_server_name().to_string()
        }).ok();
    }

    fn get_next_message(&self) -> Result<ReceiveMessage, crossbeam_channel::TryRecvError> {
        return self.get_receiver().try_recv();
    }

    fn transfer_control(&self, target: String) {
        self.get_transmitter().send(Payloads::TransferControl {
            from: self.get_server_name().to_string(),
            to: target,
        }).ok();
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        self.get_transmitter().send(Payloads::SetObserver {
            from: self.get_server_name().to_string(),
            to: target,
            is_observer: is_observer
        }).ok();
    }
}