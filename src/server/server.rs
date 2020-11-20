use crossbeam_channel::{Receiver, Sender, unbounded};
use igd::{PortMappingProtocol, SearchOptions, search_gateway};
use local_ipaddress;
use log::warn;
use retain_mut::RetainMut;
use std::{time::Duration, collections::HashMap, time::Instant, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, thread};
use laminar::{Packet, Socket, SocketEvent};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU16, Ordering::SeqCst}};
use std::str::FromStr;
use super::{Error, MAX_PUNCH_RETRIES, Payloads, get_bind_address, get_rendezvous_server, messages, util::{TransferClient}};

#[derive(Debug, Copy, Clone)]
pub enum PortForwardResult {
    GatewayNotFound,
    LocalAddrNotFound,
    AddPortError
}

struct Client {
    addr: SocketAddr,
    is_observer: bool
}

struct TransferStruct {
    session_id: Option<String>,
    clients: HashMap<String, Client>,
    // Reading/writing to UDP stream
    receiver: Receiver<SocketEvent>,
    sender: Sender<Packet>,
    // Holepunching
    rendevous_server: Option<SocketAddr>,
    clients_to_holepunch: Vec<HolePunchSession>,
    // Sending/writing to app
    server_tx: Sender<Payloads>,
    client_rx: Receiver<Payloads>,
    
    in_control: String
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

        messages::send_message(Payloads::Handshake {is_initial: true, session_id: String::new()}, session.addr.clone(), sender).ok();
        // Over retry limit, stop connection
        if session.retries > MAX_PUNCH_RETRIES {
            return false
        }
        // Reset second timer
        session.timer = Some(Instant::now());

        return true;
    });
}

fn send_to_all(except: Option<&SocketAddr>, payload: Payloads, transfer: &mut TransferStruct) {
    for (_, client) in transfer.clients.iter() {
        if let Some(except) = except {
            if client.addr == *except {continue}
        }

        messages::send_message(payload.clone(), client.addr.clone(), &mut transfer.sender).ok();
    }
}

fn handle_message(addr: SocketAddr, payload: Payloads, transfer: &mut TransferStruct) {
    let sender = &mut transfer.sender;

    let mut should_relay = true;

    match &payload {
        // Unused for server
        Payloads::InvalidName {  } => {}
        Payloads::PlayerJoined { .. } => {}
        Payloads::PlayerLeft { .. } => {}
        Payloads::SetObserver { .. } => {} 
        Payloads::PeerEstablished { .. } => {}
        // No processing needed
        Payloads::Update { .. } => {}
        // Used
        Payloads::Name { name } => {
            // Name already in use
            if transfer.clients.contains_key(name) {
                messages::send_message(Payloads::InvalidName{}, addr, sender).ok();
                return
            }

            transfer.clients.insert(name.clone(), Client {
                addr: addr.clone(),
                is_observer: false,
            });

            messages::send_message(Payloads::PlayerJoined { name: name.clone(), in_control: false, is_server: false, is_observer: false}, addr, sender).ok();

            should_relay = false;
        }
        
        Payloads::TransferControl { from: _, to } => {
            transfer.in_control = to.clone();
        }
        
        Payloads::Handshake { is_initial: _, session_id } => {
                // Incoming UDP packet from peer
            if let Some(verify_session_id) = transfer.session_id.as_ref() {
                if *session_id == *verify_session_id {
                    // TODO: add client
                    messages::send_message(Payloads::PeerEstablished {peer: addr}, transfer.rendevous_server.as_ref().unwrap().clone(), sender).ok();
                }
            }
            should_relay = false;
        }
        Payloads::HostingReceived { session_id } => {
            transfer.session_id = Some(session_id.clone());
            should_relay = false;
        }
        Payloads::AttemptConnection { peer } => {
            transfer.clients_to_holepunch.push(HolePunchSession::new(peer.clone()));
            should_relay = false;
        }
        
    }

    if should_relay {
        send_to_all(Some(&addr), payload.clone(), transfer);
    }

    transfer.server_tx.send(payload).ok();
}

fn handle_app_message(transfer: &mut TransferStruct) {
    if let Ok(payload) = transfer.client_rx.try_recv() {
        send_to_all(None, payload, transfer);
    }
}

pub struct Server {
    number_connections: Arc<AtomicU16>,
    should_stop: Arc<AtomicBool>,

    transfer: Option<Arc<Mutex<TransferStruct>>>,
    
    // Send data to peers
    client_tx: Sender<Payloads>,
    // Internally receive data to send to clients
    client_rx: Receiver<Payloads>,

    // Send data to app to receive client data
    server_tx: Sender<Payloads>,
    // Recieve data from server
    server_rx: Receiver<Payloads>,

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

    pub fn start(&mut self, is_ipv6: bool, port: u16) -> Result<(Sender<Payloads>, Receiver<Payloads>), laminar::ErrorKind> {
        // Attempt to port forward
        if let Err(e) = self.port_forward(port) {
            warn!("Could not port forward! Reason: {:?}", e)
        }
        
        let socket = Socket::bind(get_bind_address(is_ipv6))?;
        self.run(socket, None)
    }

    pub fn start_with_hole_punching(&mut self, is_ipv6: bool) -> Result<(Sender<Payloads>, Receiver<Payloads>), laminar::ErrorKind> {
        let socket = Socket::bind(get_bind_address(is_ipv6))?;
        let addr: SocketAddr = get_rendezvous_server(is_ipv6);

        // Send message to external server to obtain session ID
        messages::send_message(Payloads::Handshake {is_initial: true, session_id: String::new()}, addr.clone(), &mut socket.get_packet_sender()).ok();

        self.run(socket, Some(addr))
    }

    fn run(&mut self, socket: Socket, rendevous: Option<SocketAddr>) -> Result<(Sender<Payloads>, Receiver<Payloads>), laminar::ErrorKind> {
        let mut socket = socket;

        let transfer = Arc::new(Mutex::new(TransferStruct {
            // Holepunching
            session_id: None,
            rendevous_server: rendevous,
            clients_to_holepunch: Vec::new(),
            // Transfer
            server_tx: self.server_tx.clone(),
            client_rx: self.client_rx.clone(),
            receiver: socket.get_event_receiver(), 
            sender: socket.get_packet_sender(),
            // State
            in_control: String::new(),
            clients: HashMap::new(),
        }));
        let transfer_thread_clone = transfer.clone();

        self.transfer = Some(transfer);

        
        
        thread::spawn(move || socket.start_polling());
        thread::spawn(move || {
            loop {
                let mut transfer = transfer_thread_clone.lock().unwrap();

                let (addr, payload) = match messages::get_next_message(&mut transfer.receiver) {
                    Ok(a) => a,
                    Err(e) => match e {
                        Error::SerdeError(_) => {continue}
                        Error::ConnectionClosed(_) => {continue}
                        Error::Dummy => {continue}
                    }
                };

                handle_hole_punch(&mut transfer);
                handle_app_message(&mut transfer);
                handle_message(addr, payload, &mut transfer);
            }
        });

        Ok((self.client_tx.clone(), self.server_rx.clone()))
    }
}

impl TransferClient for Server {
    fn get_connected_count(&self) -> u16 {
        return self.number_connections.load(SeqCst);
    }

    fn stop(&self, reason: String) {
        self.should_stop.store(true, SeqCst);
    }

    fn is_server(&self) -> bool {
        true
    }

    fn stopped(&self) -> bool {
        self.should_stop.load(SeqCst)
    }

    fn get_transmitter(&self) -> &Sender<Payloads> {
        return &self.client_tx;
    }

    fn get_receiver(&self) -> &Receiver<Payloads> {
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
        
        self.client_tx.send(Payloads::TransferControl {
            from: self.get_server_name().to_string(),
            to: target,
        }).ok();
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        // Read for initial contact with other clients
        if let Some(transfer) = self.transfer.as_ref() {
            if let Some(client) = transfer.lock().unwrap().clients.get_mut(&target) {
                client.is_observer = true;
            }
        }

        self.client_tx.send(Payloads::SetObserver {
            from: self.get_server_name().to_string(),
            to: target,
            is_observer: is_observer
        });
    }

    fn get_session_id(&self) -> Option<String> {
        if let Some(transfer) = self.transfer.as_ref() {
            return transfer.lock().unwrap().session_id.clone()
        }
        return None
    }
}