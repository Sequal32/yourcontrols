use crossbeam_channel::{Receiver, Sender};
use log::info;
use serde_json::{Value, json};
use std::{fmt::Display, io::Write};

use crate::definitions::AllNeedSync;


pub trait TransferClient {
    fn get_connected_count(&self) -> u16;
    fn stop(&self, reason: String);
    fn stopped(&self) -> bool;
    fn is_server(&self) -> bool;

    fn get_transmitter(&self) -> &Sender<Value>;
    fn get_receiver(&self) -> &Receiver<ReceiveData>;
    fn get_server_name(&self) -> &str;
    // Application specific functions

    fn send_value(&self, message: Value) {
        let mut message = message;
        message["from"] = Value::String(self.get_server_name().to_string());
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

    fn transfer_control(&self, target: String) {
        self.send_value(json!({
            "type": "transfer_control",
            "target": target
        }));
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        self.send_value(json!({
            "type": "set_observer",
            "target": target,
            "is_observer": is_observer
        }));
    }

    fn send_name(&self) {
        self.send_value(json!({
            "type": "name",
            "data": self.get_server_name()
        }))
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

    pub fn add_buf(&mut self, buf: &[u8]) {
        self.buffer.extend_from_slice(buf);
    }

    pub fn try_read_string(&mut self) -> Option<String> {
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

pub fn process_message(message: &str, from: Option<String>) -> Result<ReceiveData, ParseError> {
    // Parse string into json
    let value: Value = match serde_json::from_str(message.trim()) {
        Ok(v) => v,
        Err(e) => return Err(ParseError::InvalidJson(e))
    };

    let sender: String = match from {
        Some(from) => from,
        None => match value["from"].as_str() {
            Some(from) => from.to_string(),
            None => String::new()
        }
    };

    match value["type"].as_str() {
        Some("update") => (),
        None | Some(_) => {
            if message.trim() != "" {
                info!("Data: {}", message.trim());
            }
        }
    }

    // Determine message type
    match value["type"].as_str() {
        // Parse payload into AllNeedSync
        Some("update") => match serde_json::from_value(value["data"].clone()) {
            Ok(data) => Ok(ReceiveData::Update(sender, data)),
            Err(e) => Err(ParseError::InvalidPayload(e))
        }

        Some("name") => match value["data"].as_str() {
            Some(data) => Ok(ReceiveData::Name(data.to_string())),
            None => Err(ParseError::FieldMissing("data"))
        }

        Some("transfer_control") => match value["target"].as_str() {
            Some(to) => Ok(ReceiveData::TransferControl(sender, to.to_string())),
            None => Err(ParseError::FieldMissing("target"))
        },

        Some("set_observer") => match value["target"].as_str() {
            Some(to) => Ok(ReceiveData::SetObserver(to.to_string(), value["is_observer"].as_bool().unwrap_or_default())),
            None => Err(ParseError::FieldMissing("target"))
        }

        Some("user") => match value["data"].as_str() {
            Some(name) => {
                Ok(ReceiveData::NewUser(
                    name.to_string(),
                    value["in_control"].as_bool().unwrap_or_default(),
                    value["is_observer"].as_bool().unwrap_or_default(),
                    value["is_server"].as_bool().unwrap_or_default()
                ))
            },
            None => Err(ParseError::FieldMissing("data"))
        }
        // Disconnect
        Some("invalid_name") => Ok(ReceiveData::InvalidName),

        Some("remove_user") => match value["data"].as_str() {
            Some(name) => Ok(ReceiveData::ConnectionLost(name.to_string())),
            None => Err(ParseError::FieldMissing("data"))
        }

        Some(_) => Err(ParseError::InvalidType),
        _ => Err(ParseError::FieldMissing("type")),
    }
}

// Processing message error
#[derive(Debug)]
pub enum ParseError {
    InvalidJson(serde_json::Error),
    InvalidPayload(serde_json::Error),
    FieldMissing(&'static str),
    InvalidType
}

// Various types of data to receive
pub enum ReceiveData {
    // Name
    NewConnection(String),
    // Name, is_observer, in_control, is_server
    NewUser(String, bool, bool, bool),
    // Name
    ConnectionLost(String),
    TransferStopped(TransferStoppedReason),
    // Possible types of data to receive
    Update(String, AllNeedSync),
    // From, To
    TransferControl(String, String),
    // Target, is_observer
    SetObserver(String, bool),
    Name(String),
    InvalidName
}

pub enum TransferStoppedReason {
    Requested(String),
    ConnectionLost
}

impl Display for TransferStoppedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferStoppedReason::Requested(reason) => write!(f, "{}", reason),
            TransferStoppedReason::ConnectionLost => write!(f, "Connection lost/terminated.")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_partial_reader() {
        let mut pr = PartialReader::new();
        assert_eq!(pr.try_read_string("Hello".as_bytes()), None);
        assert_eq!(pr.try_read_string("\r\nYes".as_bytes()).unwrap(), "Hello");
        assert_eq!(pr.try_read_string("\r\nYes\r\n".as_bytes()).unwrap(), "Yes");
    }
}