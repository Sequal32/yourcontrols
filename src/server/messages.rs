use std::{net::SocketAddr};

use crossbeam_channel::{Sender};
use laminar::{Packet, SocketEvent};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::definitions::AllNeedSync;

use super::SenderReceiver;

const ACK_BYTES: &[u8] = &[0x41, 0x43, 0x4b];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Payloads {
    InvalidName {},
    InvalidVersion {server_version: String},
    PlayerJoined {name: String, in_control: bool, is_server: bool, is_observer: bool},
    PlayerLeft {name: String},
    Update {data: AllNeedSync, time: f64, from: String, is_unreliable: bool},
    InitHandshake {name: String, version: String},
    TransferControl {from: String, to: String},
    SetObserver {from: String, to: String, is_observer: bool},
    // Hole punching payloads
    Handshake {session_id: String}, // With hoster
    HostingReceived {session_id: String},
    AttemptConnection {peer: SocketAddr},
    PeerEstablished {peer: SocketAddr},
}

#[derive(Debug)]
pub enum Error {
    SerdeError(serde_json::Error),
    ConnectionClosed(SocketAddr),
    ReadTimeout,
    Dummy
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeError(e)
    }
}

// Need to manually acknowledge since most of the time we don't send packets back
fn handle_packet(sender: &mut Sender<Packet>, packet: &Packet) {
    if let laminar::DeliveryGuarantee::Reliable = packet.delivery_guarantee() {
        match packet.order_guarantee() {
            laminar::OrderingGuarantee::None => {}
            laminar::OrderingGuarantee::Sequenced(stream) => {sender.send(Packet::reliable_sequenced(packet.addr(), ACK_BYTES.to_vec(), stream)).ok();},
            laminar::OrderingGuarantee::Ordered(stream) => {sender.send(Packet::reliable_ordered(packet.addr(), ACK_BYTES.to_vec(), stream)).ok();}
        };
    }
}
//
fn read_value(transfer: &mut SenderReceiver) -> Result<(SocketAddr, Value), Error>  {
    let packet = match transfer.get_receiver().try_recv() {
        Ok(event) => match event {
            SocketEvent::Packet(packet) => {
                handle_packet(transfer.get_sender(), &packet);
                packet
            },
            SocketEvent::Disconnect(addr) |
            SocketEvent::Timeout(addr) => {return Err(Error::ConnectionClosed(addr))}
            _ => {return Err(Error::Dummy)}
        }
        Err(_) => return Err(Error::ReadTimeout)
    };

    match serde_json::from_slice(packet.payload()) {
        Ok(s) => Ok((packet.addr(), s)),
        Err(e) => Err(Error::SerdeError(e))
    }
}

pub fn get_next_message(transfer: &mut SenderReceiver) -> Result<(SocketAddr, Payloads), Error> {
    let (addr, json) = read_value(transfer)?;
    Ok((addr, serde_json::from_value(json)?))
}

pub fn send_message(message: Payloads, target: SocketAddr, sender: &mut Sender<Packet>) -> Result<(), Error> {
    let payload = serde_json::to_string(&message)?.as_bytes().to_vec();

    let packet = match message {
        // Unused
        Payloads::AttemptConnection {..} |
        Payloads::HostingReceived {..} |
        // Used
        Payloads::InitHandshake {..} | 
        Payloads::PlayerJoined {..} | 
        Payloads::PlayerLeft {..} | 
        Payloads::SetObserver {..} |
        Payloads::TransferControl {..} => Packet::reliable_sequenced(target, payload, Some(3)),
        Payloads::InvalidVersion {..} | 
        Payloads::InvalidName {..} => Packet::reliable_unordered(target, payload),
        Payloads::PeerEstablished {..} |
        Payloads::Handshake {..} => Packet::unreliable(target, payload),
        Payloads::Update {is_unreliable, ..} => if is_unreliable {Packet::unreliable_sequenced(target, payload, Some(1))} else {Packet::reliable_ordered(target, payload, Some(2))}
    };

    
    match sender.try_send(packet) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:?}", e);
            Err(Error::Dummy)
        }
    }
}