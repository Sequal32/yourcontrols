use crossbeam_channel::{Receiver, Sender, unbounded};
use igd::{PortMappingProtocol, SearchOptions, search_gateway};
use local_ipaddress;
use serde_json::{Value, json};
use thread::sleep;
use std::{io::{Read}, net::IpAddr, net::Shutdown, net::TcpStream, thread, time::Duration, time::Instant};
use std::net::{TcpListener, Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering::SeqCst};
use std::{str::FromStr};
use super::{PartialReader, PartialWriter, TransferStoppedReason, process_message, util::{ReceiveData, TransferClient}};

#[derive(Debug, Copy, Clone)]
pub enum PortForwardResult {
    GatewayNotFound,
    LocalAddrNotFound,
    AddPortError
}

struct Client {
    stream: TcpStream,
    // May not be able to read all data in single loop
    reader: PartialReader,
    // May not be able to write all data in single loop
    writer: PartialWriter,
    address: IpAddr,
    name: String,
    is_observer: bool
}

struct TransferStruct {
    // Internal array of receivers to send receive data from clients
    clients: Vec<Client>,
    // Internally receive data to send to clients
    client_rx: Receiver<Value>,
    // Send data to app to receive client data
    server_tx: Sender<ReceiveData>,
    // Send data to other clients
    client_tx: Sender<Value>,
    // Server name
    name: String,
    in_control: String
}

impl TransferStruct {
    pub fn name_exists(&self, name: &str) -> bool {
        for client in self.clients.iter() {
            if name == client.name {return true}
        }
        return false;
    }

    pub fn set_observer(&mut self, name: &str, is_observer: bool) {
        for client in self.clients.iter_mut() {
            if name == client.name {
                client.is_observer = is_observer;
            }
        }
    }

    pub fn write_to_all_except(&mut self, name: &str, bytes: &[u8]) {
        for client in self.clients.iter_mut() {
            if name == client.name {continue}
            client.writer.to_write(bytes);
        }
    }
}

fn build_user_string(name: &str, is_observer: bool, has_control: bool) -> String {
    format!(r#"{{"type":"user", "data":"{}", "in_control":{}, "is_observer":{}}}{}"#, name, has_control, is_observer, "\n")
}

pub struct Server {
    number_connections: Arc<AtomicU16>,
    port_error: Option<PortForwardResult>,
    should_stop: Arc<AtomicBool>,

    transfer: Option<Arc<Mutex<TransferStruct>>>,
    
    // Send data to clients
    client_tx: Sender<Value>,
    // Internally receive data to send to clients
    client_rx: Receiver<Value>,

    // Send data to app to receive client data
    server_tx: Sender<ReceiveData>,
    // Recieve data from clients
    server_rx: Receiver<ReceiveData>,
    username: String,
}

impl Server {
    pub fn new(username: String) -> Self  {
        let (client_tx, client_rx) = unbounded();
        let (server_tx, server_rx) = unbounded();

        return Self {
            number_connections: Arc::new(AtomicU16::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
            port_error: None,
            transfer: None,
            client_rx, client_tx, server_rx, server_tx,
            username: username
        }
    }

    fn port_forward(&self, port: u16) -> Result<(), PortForwardResult> {
        let mut options = SearchOptions::default();
        options.timeout = Some(Duration::from_secs(2));

        let gateway = search_gateway(options);

        if !gateway.is_ok() {return Err(PortForwardResult::GatewayNotFound)}

        let local_addr = local_ipaddress::get();
        if !local_addr.is_some() {return Err(PortForwardResult::LocalAddrNotFound)}
        let local_addr = Ipv4Addr::from_str(local_addr.unwrap().as_str()).unwrap();

        let result = gateway.unwrap().add_port(PortMappingProtocol::TCP, port, SocketAddrV4::new(local_addr, port), 86400, "YourControl");
        if result.is_err() {return Err(PortForwardResult::AddPortError)}

        Ok(())
    }

    pub fn start(&mut self, is_ipv6: bool, port: u16) -> Result<(), std::io::Error> {
        // Attempt to port forward
        if let Err(e) = self.port_forward(port) {
            self.port_error = Some(e);
            println!("{:?}", e);
        }

        // Start listening for connections
        let bind_ip = if is_ipv6 {"::"} else {"0.0.0.0"};
        let listener = match TcpListener::bind(format!("{}:{}", bind_ip, port)) {
            Ok(listener) => listener,
            Err(n) => {return Err(n)}
        };

        // Needed to stop the server
        listener.set_nonblocking(true).ok();

        // to be used in run()
        self.transfer = Some(Arc::new(Mutex::new(
            TransferStruct {
                clients: Vec::new(),
                client_rx: self.client_rx.clone(), 
                server_tx: self.server_tx.clone(),
                client_tx: self.client_tx.clone(),
                name: self.username.clone(),
                in_control: self.username.clone()
            }
        )));

        let transfer = self.transfer.as_ref().unwrap().clone();
        let number_connections = self.number_connections.clone();
        let should_stop = self.should_stop.clone();

        // Listen for new connections
        thread::spawn(move || {
            loop {
                // Accept connection
                if let Ok((stream, addr)) = listener.accept() {
                    // Do not block as we need to iterate over all streams
                    stream.set_nonblocking(true).unwrap();

                    let mut transfer = transfer.lock().unwrap();
                    let server_name = transfer.name.clone();

                    let mut new_client = Client {
                        stream,
                        writer: PartialWriter::new(),
                        reader: PartialReader::new(),
                        address: addr.ip(),
                        name: server_name,
                        is_observer: false
                    };

                    // Identify with name
                    new_client.writer.to_write(format!(r#"{{"type":"name", "data":"{0:}"}}{1:}"#, transfer.name, "\n").as_bytes());
                    // Send server user state
                    let client_in_control = transfer.in_control.clone();
                    new_client.writer.to_write(build_user_string(&transfer.name, false, client_in_control == transfer.name).as_bytes());
                    // Iterate through all connected clients and send names
                    for client in transfer.clients.iter_mut() {
                        let in_control = client_in_control == client.name;
                        new_client.writer.to_write(build_user_string(&client.name, client.is_observer, in_control).as_bytes());
                    }
                    // Append client transfers into vector
                    transfer.clients.push(new_client);
                    // Increment number of connections and tell app
                    number_connections.fetch_add(1, SeqCst);
                }
                // Break the loop if the server's stopped
                if should_stop.load(SeqCst) {break}
                sleep(Duration::from_millis(100));
            }
        });
        

        return Ok(());
    }

    pub fn run(&mut self) {
        let transfer = self.transfer.as_ref().unwrap().clone();
        let number_connections = self.number_connections.clone();
        let should_stop = self.should_stop.clone();

        thread::spawn(move || {
            let mut timer = Instant::now();
            loop {
                let transfer = &mut transfer.lock().unwrap();
                let mut to_write = Vec::new();
                // Clients to remove 
                let mut to_drop = Vec::new();
                // Read any data from client 
                let mut next_send_string: Option<String> = match transfer.client_rx.try_recv() {
                    Ok(mut data) => {
                        data["from"] = Value::String(transfer.name.clone());
                        Some(data.to_string() + "\n")
                    },
                    Err(_) => None
                };

                // Heartbeat
                if timer.elapsed().as_secs() > 2 && next_send_string.is_none() {
                    next_send_string = Some("\n".to_string());
                    timer = Instant::now();
                }

                
                // Read incoming stream data

                for (index, client) in transfer.clients.iter_mut().enumerate() {
                    // Read buffs
                    let mut buf = [0; 1024];
                    match client.stream.read(&mut buf) {
                        // Read nothing - conenction dropped
                        Ok(0) => {
                            to_drop.push(index);
                        }
                        Ok(n) => {
                            // Append bytes to reader
                            if let Some(data_string) = client.reader.try_read_string(&buf[0..n]) {
                                // Parse payload
                                if let Ok(data) = process_message(&data_string, Some(client.name.clone())) {
                                    to_write.push((index, data, data_string));
                                }
                            }
                        }
                        
                        Err(e) => ()
                    }

                    match client.writer.write_to(&mut client.stream) {
                        Ok(_) => {}
                        // Write error - connection dropped
                        Err(e) => {
                            to_drop.push(index);
                        }
                    };
                    // Send data from app to clients
                    if let Some(data) = next_send_string.as_ref() {
                        client.writer.to_write(data.as_bytes());
                    }
                }
    
                // Send resulting incoming stream data to app
                for (client_index, data, data_string) in to_write {
                    let rebroadcast;

                    match &data {
                        ReceiveData::Name(name) => {
                            // Check that the name is not already in use
                            if transfer.name_exists(&name) || name == &transfer.name {
                                // Tell *single* client that that's an invalid name
                                let client = transfer.clients.get_mut(client_index).unwrap();
                                client.writer.to_write(concat!(r#"{"type":"invalid_name"}"#, "\n").as_bytes());
                            } else {
                                // Append address
                                let client = transfer.clients.get_mut(client_index).unwrap();
                                client.name = name.clone();
                                transfer.server_tx.send(ReceiveData::NewConnection(name.clone())).ok();
                                // Tell everyone else about the new client
                                transfer.write_to_all_except(name, build_user_string(name, false, false).as_bytes());
                            }
                            rebroadcast = false;
                        },
                        ReceiveData::TransferControl(_, to) => {
                            // Keep track of who's in control for inital state update to new clients
                            transfer.in_control = to.clone();
                            rebroadcast = true;
                        }
                        _ => {rebroadcast = true;}
                    }

                    if rebroadcast {
                        // Do not process name payload in app
                        let client = transfer.clients.get(client_index).unwrap();
                        transfer.server_tx.send(data).ok();
                        // Rebroadcast
                        let mut rebroadcast_data: Value = serde_json::from_str(&data_string).unwrap();
                        rebroadcast_data["from"] = Value::String(client.name.clone());

                        let name = client.name.clone();
                        transfer.write_to_all_except(&name, rebroadcast_data.to_string().as_bytes());
                    }
                }

                let should_stop = should_stop.load(SeqCst);
                // Shutdown all streams
                if should_stop {
                    to_drop.clear();
                    to_drop.extend(0..transfer.clients.len());
                }
    
                // Remove any connections that got dropped and tell app
                for dropping in to_drop {
                    let removed_client = transfer.clients.remove(dropping);
                    removed_client.stream.shutdown(Shutdown::Both).ok();
                    number_connections.fetch_sub(1, SeqCst);
                    
                    // TransferStopped message will take care of removing clients
                    if !should_stop {
                        transfer.server_tx.send(ReceiveData::ConnectionLost(removed_client.name.clone())).ok();
                        // Tell everyone else about client disconnect
                        transfer.client_tx.send(json!({
                            "type":"remove_user",
                            "data": removed_client.name
                        })).ok();
                    };
                }

                if should_stop {break}
                sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn get_last_port_forward_error(&self) -> &Option<PortForwardResult> {
        return &self.port_error
    }
}

impl TransferClient for Server {
    fn get_connected_count(&self) -> u16 {
        return self.number_connections.load(SeqCst);
    }

    fn stop(&self, reason: String) {
        self.server_tx.send(ReceiveData::TransferStopped(TransferStoppedReason::Requested(reason))).ok();
        self.should_stop.store(true, SeqCst);
    }

    fn is_server(&self) -> bool {
        true
    }

    fn stopped(&self) -> bool {
        self.should_stop.load(SeqCst)
    }

    fn get_transmitter(&self) -> &Sender<Value> {
        return &self.client_tx;
    }

    fn get_receiver(&self) ->& Receiver<ReceiveData> {
        return &self.server_rx;
    }

    fn get_server_name(&self) -> &str {
        return &self.username;
    }

    fn transfer_control(&self, target: String) {
        // Read for initial contact with other clients
        if let Some(transfer) = self.transfer.as_ref() {
            transfer.lock().unwrap().in_control = target.clone();
        }
        
        self.send_value(json!({
            "type": "transfer_control",
            "target": target
        }));
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        // Read for initial contact with other clients
        if let Some(transfer) = self.transfer.as_ref() {
            transfer.lock().unwrap().set_observer(&target, is_observer);
        }

        self.send_value(json!({
            "type": "set_observer",
            "target": target,
            "is_observer": is_observer
        }));
    }
}