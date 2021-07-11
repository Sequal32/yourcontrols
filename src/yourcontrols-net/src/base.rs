use crate::error::Result;
use crate::StartableNetworkObject;
use crossbeam_channel::{Receiver, Sender};
use laminar::{Config, Packet, Socket, SocketEvent};
use serde::{de::DeserializeOwned, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::{collections::HashSet, net::ToSocketAddrs};

pub const HEARTBEAT_INTERVAL: u64 = 500;

pub struct BaseSocket {
    socket: Socket,
    tx: Sender<Packet>,
    rx: Receiver<SocketEvent>,
    connections: HashSet<SocketAddr>,
}

impl StartableNetworkObject<crate::Error> for BaseSocket {
    fn start_with_bind_address(addr: impl ToSocketAddrs) -> Result<Self> {
        let socket = Socket::bind_with_config(addr, get_config())?;

        Ok(Self {
            tx: socket.get_packet_sender(),
            rx: socket.get_event_receiver(),
            socket,
            connections: HashSet::new(),
        })
    }
}

impl BaseSocket {
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
                SocketEvent::Packet(packet) => match read_bytes(packet.payload()) {
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
        let packet = payload.get_packet(addr, get_bytes(payload)?);

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

fn get_config() -> Config {
    Config {
        receive_buffer_max_size: 65536,
        heartbeat_interval: Some(Duration::from_millis(HEARTBEAT_INTERVAL)),
        ..Default::default()
    }
}

fn get_bytes(payload: &impl Serialize) -> Result<Vec<u8>> {
    #[cfg(debug_assertions)]
    return Ok(serde_json::to_vec(payload)?);
    #[cfg(not(debug_assertions))]
    return Ok(rmp_serde::to_vec(payload)?);
}

fn read_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    #[cfg(debug_assertions)]
    return Ok(serde_json::from_slice(bytes)?);
    #[cfg(not(debug_assertions))]
    return Ok(rmp_serde::from_read_ref(bytes)?);
}
