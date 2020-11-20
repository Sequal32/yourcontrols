use crossbeam_channel::{Receiver, Sender};
use log::info;
use std::{fmt::Display, net::SocketAddr};
use std::time::SystemTime;

use crate::definitions::AllNeedSync;

use super::Payloads;

pub const MAX_PUNCH_RETRIES: u8 = 5;

pub fn get_bind_address(is_ipv6: bool, port: Option<u16>) -> SocketAddr {
    let bind_string = format!("{}:{}", if is_ipv6 {"::"} else {"0.0.0.0"}, port.unwrap_or(0));
    bind_string.parse().unwrap()
}

pub fn get_rendezvous_server(is_ipv6: bool) -> SocketAddr {
    if is_ipv6 {dotenv::var("RENDEZVOUS_SERVER_V6").unwrap().parse().unwrap()} else {dotenv::var("RENDEZVOUS_SERVER_V4").unwrap().parse().unwrap()}
}

fn get_seconds() -> f64 {
    return SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64()
}

pub trait TransferClient {
    fn get_connected_count(&self) -> u16;
    fn stop(&self, reason: String);
    fn stopped(&self) -> bool;
    fn is_server(&self) -> bool;

    fn get_transmitter(&self) -> &Sender<Payloads>;
    fn get_receiver(&self) -> &Receiver<Payloads>;
    fn get_server_name(&self) -> &str;
    fn get_session_id(&self) -> Option<String>;
    // Application specific functions

    fn update(&self, data: AllNeedSync) {
        self.get_transmitter().send(Payloads::Update {
            data,
            time: get_seconds(),
            from: self.get_server_name().to_string()
        });
    }

    fn get_next_message(&self) -> Result<Payloads, crossbeam_channel::TryRecvError> {
        return self.get_receiver().try_recv();
    }

    fn transfer_control(&self, target: String) {
        self.get_transmitter().send(Payloads::TransferControl {
            from: self.get_server_name().to_string(),
            to: target,
        });
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        self.get_transmitter().send(Payloads::SetObserver {
            from: self.get_server_name().to_string(),
            to: target,
            is_observer: is_observer
        });
    }
}

// Processing message error
#[derive(Debug)]
pub enum ParseError {
    InvalidJson(serde_json::Error),
    InvalidPayload(serde_json::Error),
    FieldMissing(&'static str),
    InvalidType
}

pub enum TransferStoppedReason {
    Requested(String),
    ConnectionLost
}

impl Display for TransferStoppedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferStoppedReason::Requested(reason) => write!(f, "{}", reason),
            TransferStoppedReason::ConnectionLost => write!(f, "Connection lost/terminated.")
        }
    }
}