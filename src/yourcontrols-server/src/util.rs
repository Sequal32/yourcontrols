use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use laminar::Config;
use log::info;
use rand::prelude::SliceRandom;

const LETTERS: &[u8; 26] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const SESSION_ID_LENGTH: usize = 8;

pub fn get_random_id(length: usize) -> String {
    let mut rng = rand::thread_rng();

    let mut code = String::new();
    for _ in 0..length {
        code.push(LETTERS.choose(&mut rng).map(|&x| x as char).unwrap());
    }

    code
}

fn get_system_hour() -> u32 {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        * 0.000277778) as u32
}

struct CounterInfo {
    count: u32,
    hour: u32,
    last_request: Instant,
}

pub struct Counters {
    ip_ids: HashMap<IpAddr, String>,
    ip_request_count: HashMap<IpAddr, CounterInfo>,
    last_hour: u32,
}

impl Counters {
    pub fn new() -> Self {
        Self {
            ip_ids: HashMap::new(),
            ip_request_count: HashMap::new(),
            last_hour: get_system_hour(),
        }
    }

    pub fn get_id_for_addr(&mut self, addr: &IpAddr) -> String {
        match self.ip_ids.entry(*addr) {
            Occupied(o) => o.get().to_owned(),
            Vacant(v) => {
                let id = get_random_id(5);
                v.insert(id.clone());
                id
            }
        }
    }

    pub fn increment_request_counter(&mut self, addr: IpAddr) {
        let count = match self.ip_request_count.entry(addr) {
            Occupied(mut o) => {
                let o = o.get_mut();

                if o.hour != self.last_hour {
                    o.count = 0;
                    o.hour = get_system_hour();
                } else {
                    o.count += 1;
                }

                o.last_request = Instant::now();

                o.count
            }
            Vacant(v) => {
                v.insert(CounterInfo {
                    count: 1,
                    hour: get_system_hour(),
                    last_request: Instant::now(),
                });
                1
            }
        };

        if count >= 20 && count % 20 == 0 {
            info!(
                "{} has been sending too many payloads! Count {}",
                addr, count
            );
        }
    }

    pub fn get_request_count_for(&self, addr: &IpAddr) -> u32 {
        self.ip_request_count
            .get(addr)
            .map(|x| x.count)
            .unwrap_or(0)
    }

    pub fn get_last_request_seconds(&self, addr: &IpAddr) -> u64 {
        self.ip_request_count
            .get(addr)
            .map(|x| x.last_request.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn cleanup(&mut self) {
        self.ip_request_count.retain(|_, info| info.count > 0);
    }
}

pub fn get_socket_config(timeout: u64) -> Config {
    laminar::Config {
        heartbeat_interval: Some(Duration::from_secs(1)),
        idle_connection_timeout: Duration::from_secs(timeout),
        receive_buffer_max_size: 65536,
        max_packets_in_flight: u16::MAX,
        ..Default::default()
    }
}
