use std::{net::SocketAddr};

use crossbeam_channel::{Sender};
use laminar::{Packet, SocketEvent};
use log::warn;
use serde::{Serialize, Deserialize};
use rmp_serde;

use crate::definitions::AllNeedSync;

use super::SenderReceiver;

const ACK_BYTES: &[u8] = &[0x41, 0x43, 0x4b];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Payloads {
    InvalidName {},
    InvalidVersion {server_version: String},
    PlayerJoined {name: String, in_control: bool, is_server: bool, is_observer: bool},
    PlayerLeft {name: String},
    Update {data: AllNeedSync, from: String, is_unreliable: bool, time: f64},
    InitHandshake {name: String, version: String},
    TransferControl {from: String, to: String},
    SetObserver {from: String, to: String, is_observer: bool},
    // Ready to receive data
    Ready,
    // Hole punching payloads
    Handshake {session_id: String}, // With hoster
    HostingReceived {session_id: String},
    AttemptConnection {peer: SocketAddr},
    PeerEstablished {peer: SocketAddr},
}

#[derive(Debug)]
pub enum Error {
    ConnectionClosed(SocketAddr),
    SerdeDecodeError(rmp_serde::decode::Error),
    SerdeEncodeError(rmp_serde::encode::Error),
    ReadTimeout,
    Dummy
}

// Need to manually acknowledge since most of the time we don't send packets back
fn acknowledge_packet(sender: &mut Sender<Packet>, packet: &Packet) {
    if let laminar::DeliveryGuarantee::Reliable = packet.delivery_guarantee() {
        match packet.order_guarantee() {
            laminar::OrderingGuarantee::None => {}
            laminar::OrderingGuarantee::Sequenced(stream) => {sender.send(Packet::reliable_sequenced(packet.addr(), ACK_BYTES.to_vec(), stream)).ok();},
            laminar::OrderingGuarantee::Ordered(stream) => {sender.send(Packet::reliable_ordered(packet.addr(), ACK_BYTES.to_vec(), stream)).ok();}
        };
    }
}

//

pub fn get_next_message(transfer: &mut SenderReceiver) -> Result<(SocketAddr, Payloads), Error> {
    let packet = match transfer.get_receiver().try_recv() {
        Ok(event) => match event {
            SocketEvent::Packet(packet) => packet,
            SocketEvent::Disconnect(addr) |
            SocketEvent::Timeout(addr) => {return Err(Error::ConnectionClosed(addr))}
            _ => {return Err(Error::Dummy)}
        }
        Err(_) => return Err(Error::ReadTimeout)
    };

    // Handle acknowledgements
    if packet.payload() != ACK_BYTES {
        acknowledge_packet(transfer.get_sender(), &packet);
    }

    match rmp_serde::from_slice(packet.payload()) {
        Ok(s) => Ok((packet.addr(), s)),
        Err(e) => {
            warn!("Could not deserialize packet! Reason: {}", e);
            Err(Error::SerdeDecodeError(e))
        }
    }
}

pub fn send_message(message: Payloads, target: SocketAddr, sender: &mut Sender<Packet>) -> Result<(), Error> {
    let payload = rmp_serde::to_vec(&message).map_err(|e| Error::SerdeEncodeError(e))?;

    let packet = match message {
        // Unused
        Payloads::AttemptConnection {..} |
        Payloads::HostingReceived {..} |
        // Used
        Payloads::InitHandshake {..} | 
        Payloads::PlayerJoined {..} | 
        Payloads::PlayerLeft {..} | 
        Payloads::SetObserver {..} |
        Payloads::Ready |
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
            Err(Error::Dummy)
        }
    }
}