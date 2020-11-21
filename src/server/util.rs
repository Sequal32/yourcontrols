use crossbeam_channel::{Receiver, Sender};
use log::info;
use dns_lookup::lookup_host;
use std::{fmt::Display, net::SocketAddr, net::SocketAddrV4, net::SocketAddrV6, sync::Arc, net::IpAddr, sync::atomic::{AtomicBool, Ordering::SeqCst}, time::Duration};
use std::time::SystemTime;

use crate::definitions::AllNeedSync;

use super::Payloads;

pub const MAX_PUNCH_RETRIES: u8 = 5;
const HEARTBEAT_INTERVAL: u64 = 1;

const RENDEZVOUS_SERVER_HOSTNAME: &str = "holepunch.yourcontrols.xyz";

pub fn get_bind_address(is_ipv6: bool, port: Option<u16>) -> SocketAddr {
    let bind_string = format!("{}:{}", if is_ipv6 {"::"} else {"0.0.0.0"}, port.unwrap_or(0));
    bind_string.parse().unwrap()
}

pub fn get_rendezvous_server(is_ipv6: bool) -> Result<SocketAddr, ()> {
    for ip in lookup_host(RENDEZVOUS_SERVER_HOSTNAME).unwrap() {
         match ip {
            IpAddr::V4(ip) => if !is_ipv6 {
                return Ok(
                    SocketAddr::V4(
                        SocketAddrV4::new(ip, 5555)
                    )
                )
            }
            IpAddr::V6(ip) => if is_ipv6 {
                return Ok(
                    SocketAddr::V6(
                        SocketAddrV6::new(ip, 5555, 0, 0)
                    )
                )
            }
        }
    }
    Err(())
}

pub fn get_socket_config() -> laminar::Config {
    laminar::Config {
        heartbeat_interval: Some(Duration::from_secs(HEARTBEAT_INTERVAL)),
        ..Default::default()
    }
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