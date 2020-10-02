use std::net::IpAddr;
use crossbeam_channel::{Receiver, Sender};
use serde_json::{Value, json};
use crate::definitions::AllNeedSync;

pub trait TransferClient {
    fn get_connected_count(&self) -> u16;
    fn stop(&self);
    fn stopped(&self) -> bool;
    fn is_server(&self) -> bool;

    fn get_transmitter(&self) -> &Sender<Value>;
    fn get_receiver(&self) -> &Receiver<ReceiveData>;
    // Application specific functions

    fn send_value(&self, message: Value) {
        self.get_transmitter().send(message).ok();
    }

    fn update(&self, data: AllNeedSync) {
        self.send_value(serde_json::to_value(data).unwrap());
    }

    fn get_next_message(&self) -> Result<ReceiveData, crossbeam_channel::TryRecvError> {
        return self.get_receiver().try_recv();
    }

    fn change_control(&self, control_type: ControlTransferType) {
        match control_type {
            ControlTransferType::Take => {
                self.send_value(json!({
                    "type":"take_control"
                }));
            }
            ControlTransferType::Relieve => {
                self.send_value(json!({
                    "type":"relieve_control"
                }));
            }
            ControlTransferType::Cancel => {
                self.send_value(json!({
                    "type":"cancel_relieve"
                }));
            }
        }
    }
}

pub fn process_message(message: &str) -> Result<ReceiveData, ParseError> {
    // Parse string into json
    let value: Value = match serde_json::from_str(message) {
        Ok(v) => v,
        Err(e) => return Err(ParseError::InvalidJson(e))
    };

    // Determine message type
    match value["type"].as_str() {
        // Parse payload into AllNeedSync
        Some("update") => {
            match serde_json::from_value(value["data"].clone()) {
                Ok(data) => Ok(ReceiveData::Update(data)),
                Err(e) => Err(ParseError::InvalidPayload(e))
            }
        }
        Some("take_control") => Ok(ReceiveData::ChangeControl(ControlTransferType::Take)),
        Some("relieve_control") => Ok(ReceiveData::ChangeControl(ControlTransferType::Relieve)),
        Some("cancel_relieve") => Ok(ReceiveData::ChangeControl(ControlTransferType::Cancel)),
        Some(_) => Ok(ReceiveData::Data(value)),
        _ => Err(ParseError::FieldMissing("type")),
    }
}

// Processing message error
pub enum ParseError {
    InvalidJson(serde_json::Error),
    InvalidPayload(serde_json::Error),
    FieldMissing(&'static str),
    InvalidType
}

pub enum ControlTransferType {
    Take,
    Relieve,
    Cancel
}

// Various types of data to receive
pub enum ReceiveData {
    Data(Value),
    NewConnection(IpAddr),
    ConnectionLost(IpAddr),
    TransferStopped(String),
    // Possible types of data to receive
    Update(AllNeedSync),
    ChangeControl(ControlTransferType),
    
}