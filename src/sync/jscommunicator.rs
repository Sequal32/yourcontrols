use retain_mut::RetainMut;
use serde::{Serialize, Deserialize};
use spin_sleep::sleep;
use std::{collections::{HashMap, VecDeque}, hash::{Hash}, net::{SocketAddr, TcpListener, TcpStream}, time::Duration};
use tungstenite::{HandshakeError, Message, WebSocket, accept};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum VarType {
    Local {name: String, units: Option<String>},
    Bus {connection: u16, bus: u16, name: String},
    Interaction {name: String}
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Payloads {
    // Receive
    MultiVar {data: HashMap<String, f64>},
    SingleVar {name: String, value: f64},
    Interaction {name: String},
    Handshake {name: String, polling: bool}
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TransmitPayloads {
    // Transmit
    RequestVar {var: VarType},
    AddVar {var: VarType},
    AddMany {vars: Vec<VarType>},
    SetVar {var: VarType, value: Option<f64>},
    Clear,
    TriggerInteraction {name: String}
}

struct StreamInfo {
    stream: WebSocket<TcpStream>,
    name: String,
    polling: bool
}

// Communicates with the sim via a WebSocket
pub struct JSCommunicator {
    streams: Vec<StreamInfo>,
    listener: Option<TcpListener>,
    requested_vars: Vec<VarType>,
    current_vars: HashMap<String, f64>,
    custom_count: u16,
    incoming_payloads: VecDeque<Payloads>
}

impl JSCommunicator {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
            listener: None,
            requested_vars: Vec::new(),
            current_vars: HashMap::new(),
            custom_count: 0,
            incoming_payloads: VecDeque::new()
        }
    }

    pub fn start(&mut self) {
        if self.listener.is_some() {
            // Refresh
            self.clear_vars();
            return
        }

        let listener = TcpListener::bind("0.0.0.0:7780".parse::<SocketAddr>().unwrap()).unwrap();
        listener.set_nonblocking(true).ok();

        self.listener = Some(listener);
    }

    pub fn poll(&mut self) -> Option<Payloads> {
        self.accept_connections();
        self.read_messages();

        return self.incoming_payloads.pop_front()
    }

    fn get_poll_stream(&mut self) -> Option<&mut StreamInfo> {
        return self.streams.iter_mut().find(|x| x.polling);
    }

    fn write_payload(&mut self, payload: TransmitPayloads) {
        let is_poll = match payload {
            TransmitPayloads::RequestVar {..} | 
            TransmitPayloads::AddVar {..} |  
            TransmitPayloads::AddMany {..} |
            TransmitPayloads::SetVar {..} | 
            TransmitPayloads::Clear => true,
            _ => false
        };

        let message = Message::Text(
            serde_json::to_string(&payload).unwrap()
        );

        if is_poll {
            if let Some(info) = self.get_poll_stream() {
                info.stream.write_message(message).ok();
            }
        } else {
            for info in self.streams.iter_mut() {
                info.stream.write_message(message.clone()).ok();
            }
        }
    }

    pub fn clear_vars(&mut self) {
        self.write_payload(TransmitPayloads::Clear)
    }

    pub fn request_local_var(&mut self, var_name: String, var_units: Option<String>) {
        self.write_payload(TransmitPayloads::RequestVar {
            var: VarType::Local {
                name: var_name,
                units: var_units
            },
        })
    }

    pub fn add_bus(&mut self, connection_index: u16, bus_index: u16) -> String {
        let name = self.generate_custom_var_name();

        let var_data = VarType::Bus {
            connection: connection_index, bus: bus_index, 
            name: name.clone()
        };

        self.requested_vars.push(var_data);

        return name;
    }

    pub fn add_local_var(&mut self, var_name: String, var_units: Option<String>) {
        self.requested_vars.push(VarType::Local {
            name: var_name,
            units: var_units
        });
    }

    pub fn set(&mut self, var_name: String, var_units: Option<String>, value: Option<f64>) {
        self.write_payload(TransmitPayloads::SetVar {
            var: VarType::Local {
                name: var_name,
                units: var_units
            },
            value
        })
    }

    pub fn trigger_interaction(&mut self, interaction_name: String) {
        self.write_payload(TransmitPayloads::TriggerInteraction {
            name: interaction_name
        })
    }

    pub fn toggle_bus(&mut self, bus_index: u16, connection_index: u16, bus_name: String) {
        self.write_payload(TransmitPayloads::SetVar {
            var: VarType::Bus {
                bus: bus_index, connection: connection_index, name: bus_name
            },
            value: None
        })
    }

    pub fn get_cached_var(&self, var_name: &str) -> Option<&f64> {
        self.current_vars.get(var_name)
    }

    pub fn get_all_vars(&self) -> HashMap<String, f64> {
        self.current_vars.clone()
    }

    pub fn get_number_defined(&self) -> usize {
        self.current_vars.len()
    }

    fn generate_custom_var_name(&mut self) -> String {
        self.custom_count += 1;
        return format!("Custom{}", self.custom_count);
    }

    pub fn on_connected(&mut self) {
        self.write_payload(TransmitPayloads::AddMany {
            vars: self.requested_vars.clone()
        });
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
                                polling: false
                            });

                            break;
                        }
                        Err(HandshakeError::Interrupted(mid)) => result = mid.handshake(),
                        Err(HandshakeError::Failure(e)) => break
                    }
                    sleep(Duration::from_millis(1))
                }
            }
        }
    }

    fn record_var(&mut self, name: String, value: f64) {
        self.current_vars.insert(name, value);
    }

    fn process_payload(&mut self, payload: &Payloads) {
        match payload {
            Payloads::MultiVar { data } => {
                for (name, value) in data {
                    self.record_var(name.clone(), value.clone())
                }
            }
            Payloads::SingleVar { name, value } => {
                self.record_var(name.clone(), value.clone())
            }
            Payloads::Handshake {polling, ..} => {
                if *polling {
                    self.on_connected()
                }
            }
            _ => {}
        }
    }

    fn read_messages(&mut self) {
        let mut read_payloads = Vec::new();

        self.streams.retain_mut(|info| {

            match info.stream.read_message() {
                Ok(Message::Text(text)) => {
                    println!("{}", text);
                    match serde_json::from_str(&text) {
                        Ok(payload) => {

                            if let Payloads::Handshake {name, polling} = &payload {
                                info.name = name.clone();
                                info.polling = *polling;
                            }

                            read_payloads.push(payload);

                            return true;

                        },
                        Err(e) => {println!("BAD JSON {:?} {}", e, text)}
                    }

                }
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