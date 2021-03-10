// should probably move to its own crate
mod fragment;
pub use fragment::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Payloads {
    // Transmit to Sim
    WatchVariable {},
    WatchEvent {},
    MultiWatchVariable {},
    MultiWatchEvent {},
    ExecuteCalculator {},
    AddMapping {},
    SendIncomingValues {},

    QueueInterpolationData {},
    SetInterpolationData {},
    StopInterpolation {},
    ResetInterpolation {},

    Ping,
    // Receive from Sim
    VariableChange {},
    EventTriggered {},
    Pong,
}
