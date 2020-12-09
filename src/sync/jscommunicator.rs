use serde::{Serialize, Deserialize};
use spin_sleep::sleep;
use std::{borrow::Cow, collections::{HashMap}, hash::{Hash}, net::{SocketAddr, TcpListener, TcpStream}, time::Duration};
use tungstenite::{self, HandshakeError, Message, WebSocket, accept, protocol::{CloseFrame, frame::coding::CloseCode}};

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
    Interaction {name: String}
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TransmitPayloads {
    // Transmit
    RequestVar {var: VarType},
    AddVar {var: VarType},
    AddMany {vars: Vec<VarType>},
    SetVar {var: VarType, value: Option<f64>},
    Clear
}

// Communicates with the sim via a WebSocket
pub struct JSCommunicator {
    stream: Option<WebSocket<TcpStream>>,
    listener: Option<TcpListener>,
    requested_vars: Vec<VarType>,
    current_vars: HashMap<String, f64>,
    custom_count: u16,
}

impl JSCommunicator {
    pub fn new() -> Self {
        Self {
            stream: None,
            listener: None,
            requested_vars: Vec::new(),
            current_vars: HashMap::new(),
            custom_count: 0
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

    pub fn poll(&mut self) -> Result<Payloads, ()> {
        self.accept_connections();
        self.read_message()
    }

    fn write_payload(&mut self, payload: TransmitPayloads) {
        if let Some(stream) = self.stream.as_mut() {
            stream.write_message(
                Message::Text(
                    serde_json::to_string(&payload).unwrap()
                )
            ).ok();
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
                            println!("NEW CONNECTION");
                            self.stream = Some(stream);
                            self.on_connected();
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
            _ => {}
        }
    }

    fn read_message(&mut self) -> Result<Payloads, ()> {
        let mut should_shutdown = false;

        if let Some(stream) = self.stream.as_mut() {
            match stream.read_message() {
                Ok(Message::Text(text)) => {
                    println!("{}", text);
                    match serde_json::from_str(&text) {
                        Ok(payload) => {

                            self.process_payload(&payload);
                            return Ok(payload)

                        },
                        Err(e) => return Err(())
                    }

                }
                Ok(Message::Close(_)) => {
                    should_shutdown = true;
                }
                Err(e) => {}
                _ => {}
            }
        }

        if should_shutdown {
            self.stream = None;
        }

        Err(())
    }
}

impl Drop for JSCommunicator {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.as_mut() {
            stream.close(
                Some(
                    CloseFrame { 
                        code: CloseCode::Away, 
                        reason: Cow::Borrowed("Restarting.") 
                    }
                )
            ).ok();
        }
    }
}