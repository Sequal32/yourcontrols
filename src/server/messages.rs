use crate::definitions::AllNeedSync;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use laminar::{Metrics, Packet, Socket, SocketEvent};
use rmp_serde::{self, decode, encode};
use serde::{Serialize, Deserialize};
use std::{io, net::SocketAddr};
use zstd::block::{Compressor, Decompressor};

const COMPRESS_DICTIONARY: &[u8] = include_bytes!("compress_dict.bin");

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Payloads {
    InvalidName,
    InvalidVersion {server_version: String},
    SetHost,
    RequestHosting {self_hosted: bool},
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
    Heartbeat
}

pub enum Message {
    Payload(SocketAddr, Payloads),
    ConnectionClosed(SocketAddr),
    Metrics(SocketAddr, Metrics)
}

#[derive(Debug)]
pub enum Error {
    SerdeDecodeError(decode::Error),
    SerdeEncodeError(encode::Error),
    CompressError(io::Error),
    ReadTimeout(TryRecvError),
    NotProcessed
}

impl From<decode::Error> for Error {
    fn from(e: decode::Error) -> Self {
        Self::SerdeDecodeError(e)
    }
}

impl From<encode::Error> for Error {
    fn from(e: encode::Error) -> Self {
        Self::SerdeEncodeError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::CompressError(e)
    }
}

impl From<TryRecvError> for Error {
    fn from(e: TryRecvError) -> Self {
        Self::ReadTimeout(e)
    }
}

fn get_packet_for_message(message: &Payloads, payload_bytes: Vec<u8>, target: SocketAddr) -> Packet {
    match message {
        // Unused
        Payloads::AttemptConnection {..} |
        Payloads::HostingReceived {..} |
        Payloads::SetHost {..} |
        // Used
        Payloads::InvalidVersion {..} | 
        Payloads::InvalidName {..} => Packet::reliable_unordered(target, payload_bytes),
        Payloads::PeerEstablished {..} |
        Payloads::Handshake {..} => Packet::unreliable(target, payload_bytes),
        Payloads::InitHandshake {..} | 
        Payloads::PlayerJoined {..} | 
        Payloads::PlayerLeft {..} | 
        Payloads::SetObserver {..} |
        Payloads::Ready |
        Payloads::TransferControl {..} |
        Payloads::RequestHosting {..} | 
        Payloads::Heartbeat => Packet::reliable_ordered(target, payload_bytes, Some(2)),
        Payloads::Update {is_unreliable, ..} => if *is_unreliable {Packet::unreliable_sequenced(target, payload_bytes, Some(1))} else {Packet::reliable_ordered(target, payload_bytes, Some(2))}
    }
}

pub struct SenderReceiver {
    sender: Sender<Packet>,
    receiver: Receiver<SocketEvent>,
    compressor: Compressor,
    decompressor: Decompressor
}

impl SenderReceiver {

    pub fn from_socket(socket: &Socket) -> Self {
        Self {
            sender: socket.get_packet_sender(),
            receiver: socket.get_event_receiver(),
            compressor: Compressor::with_dict(COMPRESS_DICTIONARY.to_vec()),
            decompressor: Decompressor::with_dict(COMPRESS_DICTIONARY.to_vec())
        }
    }

    pub fn get_next_message(&mut self) -> Result<Message, Error> {
        // Receive packet
        let packet = match self.receiver.try_recv()? {
            SocketEvent::Packet(packet) => packet,
            SocketEvent::Disconnect(addr) |
            SocketEvent::Timeout(addr) => {return Ok(Message::ConnectionClosed(addr))},
            SocketEvent::Metrics(addr, metrics) => {return Ok(Message::Metrics(addr, metrics))},
            _ => {return Err(Error::NotProcessed)}
        };
    
        // Decompress
        let payload_bytes = self.decompressor.decompress(packet.payload(), 16382)?;
        // Decode to struct
        let payload = rmp_serde::from_slice(&payload_bytes)?;
        return Ok(
            Message::Payload(packet.addr(), payload)
        );
    }

    fn prepare_payload_bytes(&mut self, message: &Payloads) -> Result<Vec<u8>, Error> {
        // Struct to MessagePack
        let payload = rmp_serde::to_vec(&message)?;
        // Compress payload
        return Ok(self.compressor.compress(&payload, 0)?);
    }

    pub fn send_message(&mut self, message: Payloads, target: SocketAddr) -> Result<(), Error> {
        let payload_bytes = self.prepare_payload_bytes(&message)?;
        // Send payload
        self.sender.send(get_packet_for_message(
            &message, 
            payload_bytes,
            target
        )).ok();

        Ok(())
    }

    pub fn send_message_to_multiple(&mut self, message: Payloads, targets: Vec<SocketAddr>) -> Result<(), Error> {
        let payload_bytes = self.prepare_payload_bytes(&message)?;

        for addr in targets {
            self.sender.send(get_packet_for_message(
                &message, 
                payload_bytes.clone(),
                addr
            )).ok();
        }

        Ok(())
    }
}