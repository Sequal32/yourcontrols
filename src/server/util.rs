use std::net::IpAddr;
use serde_json::Value;
use crate::definitions::AllNeedSync;

pub trait TransferClient {
    fn get_connected_count(&self) -> u16;
    fn stop(&self);
    fn stopped(&self) -> bool;
    fn is_server(&self) -> bool;
    // Application specific functions
    fn send_value(&self, message: Value);
    fn change_control(&self, control_type: ControlTransferType);
    fn update(&self, data: AllNeedSync);
}

pub enum ControlTransferType {
    Take,
    Relieve,
    Cancel
}

pub enum ReceiveData {
    Data(Value),
    NewConnection(IpAddr),
    ConnectionLost(IpAddr),
    TransferStopped(String),
}