use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet, Socket, SocketEvent};
use rmp_serde;
use std::{collections::HashSet, net::SocketAddr, thread::spawn, time::Duration};

use crate::payloads::{get_packet_from_payload, Payloads};

pub const HEARTBEAT_INTERVAL: u64 = 500;

pub struct Server {
    tx: Sender<Packet>,
    rx: Receiver<SocketEvent>,
    connections: HashSet<SocketAddr>,
}

impl Server {
    pub fn start(port: &str) -> Result<Self, laminar::ErrorKind> {
        let mut socket = Socket::bind_with_config(
            format!("0.0.0.0:{}", port),
            laminar::Config {
                receive_buffer_max_size: 65536,
                heartbeat_interval: Some(Duration::from_millis(HEARTBEAT_INTERVAL)),
                ..Default::default()
            },
        )?;

        let tx = socket.get_packet_sender();
        let rx = socket.get_event_receiver();

        spawn(move || {
            socket.start_polling();
        });

        Ok(Self {
            tx,
            rx,
            connections: HashSet::new(),
        })
    }

    pub fn get_next_message(&mut self) -> Option<SocketEvent> {
        match self.rx.try_recv() {
            Ok(event) => {
                match &event {
                    SocketEvent::Connect(addr) => {
                        self.connections.insert(addr.clone());
                    }
                    SocketEvent::Disconnect(addr) => {
                        self.connections.remove(addr);
                    }
                    _ => {}
                };

                Some(event)
            }
            Err(_) => None,
        }
    }

    pub fn decode_packet(&self, packet: Packet) -> Result<Payloads, rmp_serde::decode::Error> {
        rmp_serde::from_read_ref(packet.payload())
    }

    pub fn send_payload(&self, addr: SocketAddr, payload: Payloads) {
        self.tx.send(get_packet_from_payload(addr, payload)).ok();
    }

    pub fn send_to_all_except(&self, self_addr: SocketAddr, payload: Payloads) {
        for addr in self.connections.iter() {
            let addr = addr.clone();
            if addr == self_addr {
                continue;
            }
            self.send_payload(addr, payload.clone())
        }
    }

    pub fn send_to_multiple<I>(&self, addrs: I, payload: Payloads)
    where
        I: IntoIterator<Item = SocketAddr>,
    {
        for addr in addrs {
            self.send_payload(addr, payload.clone())
        }
    }
}
