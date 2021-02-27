use serde_repr::{Serialize_repr, Deserialize_repr};

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum ConnectionState {
    Connected = 0,
    Connecting = 1,
    Disconnected = 2
}