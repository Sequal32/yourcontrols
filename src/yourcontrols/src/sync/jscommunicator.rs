use log::{error, info};
use retain_mut::RetainMut;
use serde::{Deserialize, Serialize};
use spin_sleep::sleep;
use std::{
    collections::VecDeque,
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};
use tungstenite::{accept, HandshakeError, Message, WebSocket};

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Payloads {
    // Receive
    Interaction { name: String },
    Handshake { name: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TransmitPayloads {
    // Transmit
}

struct StreamInfo {
    stream: WebSocket<TcpStream>,
    name: String,
}

// Communicates with the sim via a WebSocket
pub struct JSCommunicator {
    streams: Vec<StreamInfo>,
    listener: Option<TcpListener>,
    incoming_payloads: VecDeque<Payloads>,
}

impl JSCommunicator {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
            listener: None,
            incoming_payloads: VecDeque::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), io::Error> {
        if self.listener.is_some() {
            return Ok(());
        }

        let listener = TcpListener::bind("0.0.0.0:7780".parse::<SocketAddr>().unwrap())?;
        listener.set_nonblocking(true).ok();

        self.listener = Some(listener);

        Ok(())
    }

    pub fn poll(&mut self) -> Option<Payloads> {
        self.accept_connections();
        self.read_messages();

        return self.incoming_payloads.pop_front();
    }

    #[allow(dead_code)]
    fn write_payload(&mut self, payload: TransmitPayloads) {
        let message = Message::Text(serde_json::to_string(&payload).unwrap());

        for info in self.streams.iter_mut() {
            info.stream.write_message(message.clone()).ok();
        }
    }

    fn accept_connections(&mut self) {
        if let Some(listener) = self.listener.as_mut() {
            if let Ok((stream, _)) = listener.accept() {
                // Keep trying to handshake
                let mut result = accept(stream);
                loop {
                    match result {
                        Ok(stream) => {
                            self.streams.push(StreamInfo {
                                stream: stream,
                                name: String::new(),
                            });

                            break;
                        }
                        Err(HandshakeError::Interrupted(mid)) => result = mid.handshake(),
                        Err(HandshakeError::Failure(_)) => break,
                    }
                    sleep(Duration::from_millis(1))
                }
            }
        }
    }

    fn process_payload(&mut self, payload: &Payloads) {
        match payload {
            _ => {}
        }
    }

    fn read_messages(&mut self) {
        let mut read_payloads = Vec::new();

        self.streams.retain_mut(|info| {
            match info.stream.read_message() {
                Ok(Message::Text(text)) => match serde_json::from_str(&text) {
                    Ok(payload) => {
                        if let Payloads::Handshake { name } = &payload {
                            info!("[JS] Panel gauge connected: {}", name);
                            info.name = name.clone();
                        }

                        read_payloads.push(payload);

                        return true;
                    }
                    Err(e) => error!(
                        "[JS] Error deserializing data! Data: {} Reason: {}",
                        text, e
                    ),
                },
                Ok(Message::Close(_)) => {
                    return false;
                }
                Err(_) => {}
                _ => {}
            }

            return true;
        });

        for payload in read_payloads {
            self.process_payload(&payload);
            self.incoming_payloads.push_back(payload);
        }
    }
}
