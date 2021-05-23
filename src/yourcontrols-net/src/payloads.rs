use std::net::SocketAddr;

use crate::base::Payload;
use laminar::Packet;
use serde::{Deserialize, Serialize};
use yourcontrols_types::ChangedDatum;

#[derive(Serialize, Deserialize)]
pub enum MainPayloads {
    // Handshake payloads
    Hello { session_id: String, version: String },
    RequestSession { self_hosted: bool },
    SessionDetails { session_id: String },
    AttemptConnection { public_ip: String, local_ip: String }, // Base64 encoded strings
    InvalidSession,
    InvalidVersion,
    // Main Game Payloads
    UpdateReliable { changed: Vec<ChangedDatum> },
}

impl Payload for MainPayloads {
    // Handshake payloads
    fn get_packet(&self, addr: SocketAddr, bytes: Vec<u8>) -> laminar::Packet {
        match self {
            MainPayloads::Hello { .. }
            | MainPayloads::RequestSession { .. }
            | MainPayloads::SessionDetails { .. }
            | MainPayloads::AttemptConnection { .. }
            | MainPayloads::InvalidSession
            | MainPayloads::InvalidVersion
            | MainPayloads::UpdateReliable { .. } => Packet::reliable_ordered(addr, bytes, None),
        }
    }
}
