use std::net::SocketAddr;

use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet, SocketEvent};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::definitions::AllNeedSync;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Payloads {
    Name {name: String},
    InvalidName {},
    PlayerJoined {name: String, in_control: bool, is_server: bool, is_observer: bool},
    PlayerLeft {name: String},
    Update {data: AllNeedSync, time: f64, from: String},
    TransferControl {from: String, to: String},
    SetObserver {from: String, to: String, is_observer: bool},
    // Hole punching payloads
    Handshake {is_initial: bool, session_id: String},
    HostingReceived {session_id: String},
    AttemptConnection {peer: SocketAddr},
    PeerEstablished {peer: SocketAddr},
}

#[derive(Debug)]
pub enum Error {
    SerdeError(serde_json::Error),
    ConnectionClosed(SocketAddr),
    Dummy
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeError(e)
    }
}
//
fn read_value(receiver: &mut Receiver<SocketEvent>) -> Result<(SocketAddr, Value), Error>  {

    let packet = match receiver.recv() {
        Ok(event) => match event {
            SocketEvent::Packet(packet) => packet,
            SocketEvent::Connect(_) => {return Err(Error::Dummy)}
            SocketEvent::Timeout(addr) | 
            SocketEvent::Disconnect(addr) => {return Err(Error::ConnectionClosed(addr))}
        }
        Err(_) => {return Err(Error::Dummy)}
    };

    match serde_json::from_slice(packet.payload()) {
        Ok(s) => Ok((packet.addr(), s)),
        Err(e) => Err(Error::SerdeError(e))
    }
}

pub fn get_next_message(receiver: &mut Receiver<SocketEvent>) -> Result<(SocketAddr, Payloads), Error> {
    let (addr, json) = read_value(receiver)?;
    Ok((addr, serde_json::from_value(json)?))
}

pub fn send_message(message: Payloads, target: SocketAddr, sender: &mut Sender<Packet>) -> Result<(), Error> {
    let payload = serde_json::to_string(&message)?.as_bytes().to_vec();

    let packet = match message {
        Payloads::Name {..} | 
        Payloads::InvalidName {..} | 
        Payloads::PlayerJoined {..} | 
        Payloads::PlayerLeft {..} | 
        Payloads::SetObserver {..} |
        Payloads::Handshake {..} |
        Payloads::HostingReceived {..} |
        Payloads::AttemptConnection {..} |
        Payloads::PeerEstablished {..} |
        Payloads::TransferControl {..} => Packet::reliable_unordered(target, payload),
        Payloads::Update {..} => Packet::unreliable(target, payload)
    };

    
    match sender.send(packet) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:?}", e);
            Err(Error::Dummy)
        }
    }
}