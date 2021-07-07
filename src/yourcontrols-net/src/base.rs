use crate::error::Result;
use crossbeam_channel::{Receiver, Sender};
use laminar::{Config, Packet, Socket, SocketEvent};
use rmp_serde;
use serde::{de::DeserializeOwned, Serialize};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::{Duration, Instant};
use std::{collections::HashSet, net::ToSocketAddrs};

pub const HEARTBEAT_INTERVAL: u64 = 500;

fn get_config() -> Config {
    Config {
        receive_buffer_max_size: 65536,
        heartbeat_interval: Some(Duration::from_millis(HEARTBEAT_INTERVAL)),
        ..Default::default()
    }
}

pub struct BaseSocket {
    socket: Socket,
    tx: Sender<Packet>,
    rx: Receiver<SocketEvent>,
    connections: HashSet<SocketAddr>,
}

impl BaseSocket {
    pub fn start() -> Result<Self> {
        Self::start_with_port(0)
    }

    pub fn start_with_port(port: u16) -> Result<Self> {
        Self::start_with_bind_address(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            port,
        )))
    }

    pub fn start_with_bind_address(address: impl ToSocketAddrs) -> Result<Self> {
        let socket = Socket::bind_with_config(address, get_config())?;

        Ok(Self {
            tx: socket.get_packet_sender(),
            rx: socket.get_event_receiver(),
            socket,
            connections: HashSet::new(),
        })
    }

    pub fn get_address(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }

    pub fn poll<T>(&mut self) -> Vec<Message<T>>
    where
        T: DeserializeOwned,
    {
        self.socket.manual_poll(Instant::now());
        self.get_messages()
    }

    pub fn poll_no_receive(&mut self) {
        self.socket.manual_poll(Instant::now());
    }

    fn get_messages<T>(&mut self) -> Vec<Message<T>>
    where
        T: DeserializeOwned,
    {
        let mut buffer = Vec::new();

        while let Ok(event) = self.rx.try_recv() {
            match event {
                SocketEvent::Connect(addr) => {
                    self.connections.insert(addr);
                    buffer.push(Message::NewConnection(addr));
                }
                SocketEvent::Timeout(addr) => {
                    self.connections.remove(&addr);
                    buffer.push(Message::LostConnection(addr));
                }
                SocketEvent::Packet(packet) => match rmp_serde::from_read_ref(packet.payload()) {
                    Ok(p) => buffer.push(Message::Payload(p, packet.addr())),
                    _ => continue,
                },
                _ => {}
            };
        }

        buffer
    }

    pub fn send_to<S>(&self, addr: SocketAddr, payload: &S) -> Result<()>
    where
        S: Serialize + Payload,
    {
        let packet = payload.get_packet(addr, rmp_serde::to_vec(payload)?);

        self.tx.send(packet).ok();

        Ok(())
    }

    pub fn send_to_all_except<S>(&self, self_addr: SocketAddr, payload: &S) -> Result<()>
    where
        S: Serialize + Payload,
    {
        for addr in self.connections.iter() {
            if *addr == self_addr {
                continue;
            }
            self.send_to(*addr, payload)?
        }

        Ok(())
    }

    pub fn send_to_multiple<I, S>(&self, addrs: I, payload: &S) -> Result<()>
    where
        I: IntoIterator<Item = SocketAddr>,
        S: Serialize + Payload,
    {
        for addr in addrs {
            self.send_to(addr, payload)?
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Message<T> {
    NewConnection(SocketAddr),
    LostConnection(SocketAddr),
    Payload(T, SocketAddr),
}

pub trait Payload {
    fn get_packet(&self, addr: SocketAddr, bytes: Vec<u8>) -> Packet;
}
