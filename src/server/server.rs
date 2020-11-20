use crossbeam_channel::{Receiver, Sender, unbounded};
use igd::{PortMappingProtocol, SearchOptions, search_gateway};
use local_ipaddress;
use log::{warn};
use retain_mut::RetainMut;
use serde_json::{Value, json};
use spin_sleep::sleep;
use std::{collections::HashMap, io::{Read}, net::IpAddr, net::Shutdown, net::SocketAddrV6, net::TcpStream, net::{SocketAddr, UdpSocket}, thread, time::Duration, time::Instant};
use std::net::{TcpListener, Ipv4Addr, SocketAddrV4};
use laminar::{Packet, Socket, SocketEvent};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering::SeqCst};
use std::{str::FromStr};
use super::{Payloads, TransferStoppedReason, messages, process_message, util::{ReceiveData, TransferClient}};

const MAX_PUNCH_RETRIES: u8 = 5;

#[derive(Debug, Copy, Clone)]
pub enum PortForwardResult {
    GatewayNotFound,
    LocalAddrNotFound,
    AddPortError
}

struct Client {
    stream: TcpStream,
    name: String,
    is_observer: bool
}

struct TransferStruct {
    session_id: Option<String>,
    clients: HashMap<SocketAddr, Option<Client>>,
    // Reading/writing to UDP stream
    receiver: Receiver<SocketEvent>,
    sender: Sender<Packet>,
    // Holepunching
    rendevous_server: Option<SocketAddr>,
    clients_to_holepunch: Vec<HolePunchSession>,
}

struct HolePunchSession {
    addr: SocketAddr,
    timer: Option<Instant>,
    retries: u8
}

impl HolePunchSession {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr: addr,
            timer: None,
            retries: 0,
        }
    }
}

fn handle_hole_punch(transfer: &mut TransferStruct) {
    let sender = &mut transfer.sender;
    let sessions = &mut transfer.clients_to_holepunch;

    sessions.retain_mut(|session| {
    // Send a message every second
        if session.timer.is_some() && session.timer.as_ref().unwrap().elapsed().as_secs() < 1 {return true}

        messages::send_message(Payloads::Handshake {is_initial: true, session_id: String::new()}, session.addr.clone(), sender);
        // Over retry limit, stop connection
        if session.retries > MAX_PUNCH_RETRIES {
            return false
        }
        // Reset second timer
        session.timer = Some(Instant::now());

        return true;
    });
}

fn handle_message(addr: SocketAddr, payload: Payloads, transfer: &mut TransferStruct) {
    let sender = &mut transfer.sender;
    match payload {
        Payloads::Name { name } => {}
        Payloads::InvalidName {  } => {}
        Payloads::PlayerJoined { name, in_control, is_server, is_observer } => {}
        Payloads::PlayerLeft { name } => {}
        Payloads::Update { data, time, from } => {}
        Payloads::TransferControl { from, to } => {}
        Payloads::SetObserver { from, to } => {}
        Payloads::Handshake { is_initial, session_id } => {
                // Incoming UDP packet from peer
            if let Some(verify_session_id) = transfer.session_id.as_ref() {
                if session_id == *verify_session_id {
                    // TODO: add client
                    transfer.clients.insert(addr.clone(), None);
                    messages::send_message(Payloads::PeerEstablished {peer: addr}, transfer.rendevous_server.as_ref().unwrap().clone(), sender).ok();
                }
            }
        }
        Payloads::HostingReceived { session_id } => {
            transfer.session_id = Some(session_id)
        }
        Payloads::AttemptConnection { peer } => {
            transfer.clients_to_holepunch.push(HolePunchSession::new(peer));
        }
        
        Payloads::PeerEstablished { peer } => {}
    }
}

fn get_rendezvous_server(is_ipv6: bool) -> SocketAddr {
    if is_ipv6 {dotenv::var("RENDEZVOUS_SERVER_V6").unwrap().parse().unwrap()} else {dotenv::var("RENDEZVOUS_SERVER_V4").unwrap().parse().unwrap()}
}

pub struct Server {
    number_connections: Arc<AtomicU16>,
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
            client_rx, client_tx, server_rx, server_tx,
            transfer: None,
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

        let result = gateway.unwrap().add_port(PortMappingProtocol::UDP, port, SocketAddrV4::new(local_addr, port), 86400, "YourControls");
        if result.is_err() {return Err(PortForwardResult::AddPortError)}

        Ok(())
    }

    pub fn start(&mut self, is_ipv6: bool, port: u16) -> Result<(), laminar::ErrorKind> {
        // Attempt to port forward
        if let Err(e) = self.port_forward(port) {
            warn!("Could not port forward! Reason: {:?}", e)
        }
        
        let bind_addr = format!("{}:{}", if is_ipv6 {"::"} else {"0.0.0.0"}, port);
        let socket = Socket::bind(bind_addr)?;

        self.run(socket, None)
    }

    pub fn start_with_hole_punching(&mut self, is_ipv6: bool) -> Result<(), laminar::ErrorKind> {
        let socket = Socket::bind(if is_ipv6 {":::0"} else {"0.0.0.0:0"})?;
        let addr: SocketAddr = get_rendezvous_server(is_ipv6);

        // Send message to external server to obtain session ID
        messages::send_message(Payloads::Handshake {is_initial: true, session_id: String::new()}, addr.clone(), &mut socket.get_packet_sender()).ok();

        self.run(socket, Some(addr))
    }

    fn run(&mut self, socket: Socket, rendevous: Option<SocketAddr>) -> Result<(), laminar::ErrorKind> {
        let mut socket = socket;

        let transfer = Arc::new(Mutex::new(TransferStruct {
            session_id: None,
            rendevous_server: rendevous,
            receiver: socket.get_event_receiver(), 
            sender: socket.get_packet_sender(),
            clients: HashMap::new(),
            clients_to_holepunch: Vec::new(),
        }));
        let transfer_thread_clone = transfer.clone();

        self.transfer = Some(transfer);

        
        thread::spawn(move || socket.start_polling());
        thread::spawn(move || {
            loop {
                let mut transfer = transfer_thread_clone.lock().unwrap();

                let (addr, payload) = match messages::get_next_message(&mut transfer.receiver) {
                    Ok(a) => a,
                    Err(_) => {
                        // TODO: handle break
                        continue;
                    }
                };

                handle_hole_punch(&mut transfer);
                handle_message(addr, payload, &mut transfer);

                sleep(Duration::from_millis(10))
            }
        });

        Ok(())
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
        // if let Some(transfer) = self.transfer.as_ref() {
        //     transfer.lock().unwrap().in_control = target.clone();
        // }
        
        // self.send_value(json!({
        //     "type": "transfer_control",
        //     "target": target
        // }));
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        // Read for initial contact with other clients
        // if let Some(transfer) = self.transfer.as_ref() {
        //     transfer.lock().unwrap().set_observer(&target, is_observer);
        // }

        // self.send_value(json!({
        //     "type": "set_observer",
        //     "target": target,
        //     "is_observer": is_observer
        // }));
    }

    fn get_session_id(&self) -> Option<String> {
        if let Some(transfer) = self.transfer.as_ref() {
            return transfer.lock().unwrap().session_id.clone()
        }
        return None
    }
}