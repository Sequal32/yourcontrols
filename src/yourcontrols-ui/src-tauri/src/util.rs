use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum ConnectionState {
    Connected = 0,
    Connecting = 1,
    Disconnected = 2,
}
