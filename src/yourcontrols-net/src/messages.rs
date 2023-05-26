use crossbeam_channel::{Receiver, Sender};
use laminar::{Metrics, Packet, Socket, SocketEvent};
use rmp_serde::{self};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Instant};
use yourcontrols_types::AllNeedSync;
use zstd::bulk::{Compressor, Decompressor};

use yourcontrols_types::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Payloads {
    InvalidName,
    InvalidVersion {
        server_version: String,
    },
    AircraftDefinition {
        bytes: Box<[u8]>,
    },
    SetHost,
    RequestHosting {
        self_hosted: bool,
        local_endpoint: Option<SocketAddr>,
    },
    ConnectionDenied {
        reason: String,
    },
    PlayerJoined {
        name: String,
        in_control: bool,
        is_server: bool,
        is_observer: bool,
    },
    PlayerLeft {
        name: String,
    },
    Update {
        data: AllNeedSync,
        from: String,
        is_unreliable: bool,
        time: f64,
    },
    InitHandshake {
        name: String,
        version: String,
    },
    TransferControl {
        from: String,
        to: String,
    },
    SetObserver {
        from: String,
        to: String,
        is_observer: bool,
    },
    SetSelfObserver {
        name: String,
    },
    // Ready to receive data
    Ready,
    // Hole punching payloads
    RendezvousHandshake {
        session_id: String,
        local_endpoint: Option<SocketAddr>,
    },
    Handshake {
        session_id: String,
    }, // With hoster
    HostingReceived {
        session_id: String,
    },
    AttemptConnection {
        peers: Vec<SocketAddr>,
    },
    AttemptHosterConnection {
        peer: SocketAddr,
    },
    PeerEstablished {
        peer: SocketAddr,
    },
    Heartbeat,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PayloadWrapper {
    pub data: Vec<u8>,
    pub size: usize, // used for decompression buffer size
}

pub enum Message {
    Payload(SocketAddr, Payloads),
    ConnectionClosed(SocketAddr),
    Metrics(SocketAddr, Metrics),
}

fn get_packet_for_message(
    message: &Payloads,
    payload_bytes: Vec<u8>,
    target: SocketAddr,
) -> Packet {
    match message {
        // Unused
        Payloads::AttemptConnection {..} |
        Payloads::AttemptHosterConnection {..} |
        Payloads::HostingReceived {..} |
        Payloads::SetHost {..} |
        Payloads::ConnectionDenied {..} |
        // Used
        Payloads::InvalidVersion {..} |
        Payloads::Heartbeat {..} |
        Payloads::SetSelfObserver { .. } |
        Payloads::InvalidName {..} => Packet::reliable_unordered(target, payload_bytes),
        Payloads::PeerEstablished {..} |
        Payloads::RendezvousHandshake  {..} |
        Payloads::Handshake {..} => Packet::unreliable(target, payload_bytes),
        Payloads::InitHandshake {..} |
        Payloads::PlayerJoined {..} |
        Payloads::PlayerLeft {..} |
        Payloads::SetObserver {..} |
        Payloads::Ready |
        Payloads::TransferControl {..} |
        Payloads::AircraftDefinition {..}  |
        Payloads::RequestHosting {..} => Packet::reliable_ordered(target, payload_bytes, Some(1)),
        Payloads::Update {is_unreliable, ..} => if *is_unreliable {Packet::unreliable_sequenced(target, payload_bytes, Some(0))} else {Packet::reliable_ordered(target, payload_bytes, Some(0))}
    }
}

fn get_compression_level_for_message(msg: &Payloads) -> i32 {
    match msg {
        Payloads::AircraftDefinition { .. } => 22,
        _ => 0,
    }
}

pub struct SenderReceiver {
    socket: Socket,
    sender: Sender<Packet>,
    receiver: Receiver<SocketEvent>,
    compressor: Compressor<'static>,
    decompressor: Decompressor<'static>,
}

impl SenderReceiver {
    pub fn from_socket(socket: Socket) -> Self {
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();

        Self {
            socket,
            sender,
            receiver,
            compressor: Compressor::new(0).unwrap(),
            decompressor: Decompressor::new().unwrap(),
        }
    }

    pub fn get_next_message(&mut self) -> Result<Message, Error> {
        // Receive packet
        let packet = match self.receiver.try_recv()? {
            SocketEvent::Packet(packet) => packet,
            SocketEvent::Timeout(addr) => return Ok(Message::ConnectionClosed(addr)),
            SocketEvent::Metrics(addr, metrics) => return Ok(Message::Metrics(addr, metrics)),
            _ => return Err(Error::NotProcessed),
        };

        // Decode wrapper struct
        let wrapper: PayloadWrapper = rmp_serde::from_slice(packet.payload())?;

        // Decompress
        let payload_bytes = self.decompressor.decompress(&wrapper.data, wrapper.size)?;

        // Decode to struct
        let payload = rmp_serde::from_slice(&payload_bytes)?;
        Ok(Message::Payload(packet.addr(), payload))
    }

    pub fn poll(&mut self) {
        self.socket.manual_poll(Instant::now());
    }

    fn prepare_payload_bytes(&mut self, message: &Payloads) -> Result<Vec<u8>, Error> {
        // Struct to MessagePack
        let payload_bytes = rmp_serde::to_vec(&message)?;

        // Compress
        self.compressor
            .set_compression_level(get_compression_level_for_message(message))?;

        let compressed = self.compressor.compress(&payload_bytes)?;

        // Wrap
        let wrapper = PayloadWrapper {
            data: compressed,
            size: payload_bytes.len(),
        };

        // Serialize
        Ok(rmp_serde::to_vec(&wrapper)?)
    }

    pub fn send_message(&mut self, message: Payloads, target: SocketAddr) -> Result<(), Error> {
        let payload_bytes = self.prepare_payload_bytes(&message)?;
        // Send payload
        self.sender
            .send(get_packet_for_message(&message, payload_bytes, target))
            .ok();

        Ok(())
    }

    pub fn send_message_to_multiple(
        &mut self,
        message: Payloads,
        targets: Vec<SocketAddr>,
    ) -> Result<(), Error> {
        let payload_bytes = self.prepare_payload_bytes(&message)?;

        for addr in targets {
            self.sender
                .send(get_packet_for_message(
                    &message,
                    payload_bytes.clone(),
                    addr,
                ))
                .ok();
        }

        Ok(())
    }
}
