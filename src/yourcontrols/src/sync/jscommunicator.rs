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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JSPayloads {
    // Receive
    Interaction { name: String },
    Handshake { name: String },
    Input { id: String, value: String },
}

pub struct JSMessage {
    pub payload: JSPayloads,
    pub instrument_name: String,
}

struct StreamInfo {
    stream: WebSocket<TcpStream>,
    name: String,
}

// Communicates with the sim via a WebSocket
pub struct JSCommunicator {
    streams: Vec<StreamInfo>,
    listener: Option<TcpListener>,
    incoming_payloads: VecDeque<JSMessage>,
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

    pub fn poll(&mut self) -> Option<JSMessage> {
        self.accept_connections();
        self.read_messages();

        return self.incoming_payloads.pop_front();
    }

    fn write_message_to_instrument(&mut self, message: Message, instrument: &str) {
        if let Some(info) = self.streams.iter_mut().find(|x| x.name == instrument) {
            info.stream.write_message(message).ok();
        }
    }

    fn write_message_to_all(&mut self, message: Message) {
        for info in self.streams.iter_mut() {
            info.stream.write_message(message.clone()).ok();
        }
    }

    pub fn write_payload(&mut self, payload: JSPayloads, instrument: Option<&str>) {
        let message = Message::Text(serde_json::to_string(&payload).unwrap());

        if let Some(instrument) = instrument {
            self.write_message_to_instrument(message, instrument);
        } else {
            self.write_message_to_all(message);
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

    fn read_messages(&mut self) {
        let incoming_payloads = &mut self.incoming_payloads;

        self.streams.retain_mut(|info| {
            match info.stream.read_message() {
                Ok(Message::Text(text)) => match serde_json::from_str(&text) {
                    Ok(payload) => {
                        match &payload {
                            JSPayloads::Handshake { name } => {
                                info!("[JS] Panel gauge connected: {}", name);
                                info.name = name.clone();
                            }
                            _ => {}
                        }

                        incoming_payloads.push_back(JSMessage {
                            payload,
                            instrument_name: info.name.clone(),
                        });

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
                _ => {}
            }

            return true;
        });
    }
}
