use crossbeam_channel::{Receiver, Sender};
use log::info;
use std::{fmt::Display, net::SocketAddr, sync::Arc, sync::atomic::{AtomicBool, Ordering::SeqCst}};
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
    fn get_stop_atomic(&self) -> &Arc<AtomicBool>;
    fn get_last_stop_reason(&mut self) -> &mut Option<String>;
    fn is_server(&self) -> bool;

    fn get_transmitter(&self) -> &Sender<Payloads>;
    fn get_receiver(&self) -> &Receiver<Payloads>;
    fn get_server_name(&self) -> &str;
    fn get_session_id(&self) -> Option<String>;
    // Application specific functions
    fn stop(&mut self, reason: String) {
        self.get_stop_atomic().store(true, SeqCst);
        *self.get_last_stop_reason() = Some(reason)
    }

    fn stopped(&mut self) -> (bool, Option<&String>) {
        if self.get_stop_atomic().load(SeqCst) {
            (true, self.get_last_stop_reason().as_ref())
        } else {
            (false, None)
        }
    }

    fn update(&self, data: AllNeedSync) {
        self.get_transmitter().send(Payloads::Update {
            data,
            time: get_seconds(),
            from: self.get_server_name().to_string()
        }).ok();
    }

    fn get_next_message(&self) -> Result<Payloads, crossbeam_channel::TryRecvError> {
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