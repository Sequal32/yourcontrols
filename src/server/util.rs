use std::{io::Write, net::IpAddr};
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
        self.send_value(json!({
            "type":"update",
            "data":data
        }));
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
            },
            ControlTransferType::Confirm => {
                self.send_value(json!({
                    "type":"confirm_relieve"
                }));
            }
        }
    }
}

pub struct PartialReader {
    buffer: Vec<u8>,
}

impl PartialReader {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new()
        }
    }

    pub fn try_read_string(&mut self, buf: &[u8]) -> Option<String> {
        self.buffer.extend_from_slice(buf);
        
        if let Some(index) = self.buffer.iter().position(|&x| x == 0x0a) {
            let result_string = String::from_utf8(self.buffer[0..index + 1].to_vec()).unwrap();
            self.buffer.drain(0..index + 1);
            return Some(result_string);
        } else {
            return None
        }
    }
}

pub struct PartialWriter {
    buffer: Vec<u8>,
}

impl PartialWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn to_write(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn write_to(&mut self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        if self.buffer.len() == 0 {return Ok(())}

        match writer.write(self.buffer.as_slice()) {
            Ok(bytes_written) => {
                self.buffer.drain(0..bytes_written);
                Ok(())
            }
            Err(e) => Err(e)
        }
    }
}

pub fn process_message(message: &str) -> Result<ReceiveData, ParseError> {
    // Parse string into json
    let value: Value = match serde_json::from_str(message.trim()) {
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
        Some("confirm_relieve") => Ok(ReceiveData::ChangeControl(ControlTransferType::Confirm)),
        Some(_) => Err(ParseError::InvalidType),
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
    Cancel,
    Confirm
}

// Various types of data to receive
pub enum ReceiveData {
    NewConnection(IpAddr),
    ConnectionLost(IpAddr),
    TransferStopped(TransferStoppedReason),
    // Possible types of data to receive
    Update(AllNeedSync),
    ChangeControl(ControlTransferType),
    
}

pub enum TransferStoppedReason {
    Requested,
    ConnectionLost
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_partial_reader() {
        let mut pr = PartialReader::new();
        assert_eq!(pr.try_read_string("Hello".as_bytes()), None);
        assert_eq!(pr.try_read_string("\nYes".as_bytes()).unwrap(), "Hello");
        assert_eq!(pr.try_read_string("\nYes\n".as_bytes()).unwrap(), "Yes");
    }
}