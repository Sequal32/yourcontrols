use crossbeam_channel::{Receiver, Sender, unbounded};
use igd::{PortMappingProtocol, SearchOptions, search_gateway};
use local_ipaddress;
use log::warn;
use retain_mut::RetainMut;
use spin_sleep::sleep;
use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}, thread, time::Duration, time::Instant};
use laminar::{Packet, Socket, SocketEvent};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU16, Ordering::SeqCst}};
use std::str::FromStr;
use super::{Error, MAX_PUNCH_RETRIES, Payloads, get_bind_address, get_rendezvous_server, get_socket_config, messages, util::{TransferClient}};

#[derive(Debug)]
pub enum PortForwardResult {
    GatewayNotFound(igd::SearchError),
    LocalAddrNotFound,
    AddPortError(igd::AddPortError)
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
    
    in_control: String,
    should_stop: Arc<AtomicBool>,
}

impl TransferStruct {
    fn get_sender(&mut self) -> &mut Sender<Packet> {
        &mut self.sender
    }

    
    fn send_to_all(&mut self, except: Option<&SocketAddr>, payload: Payloads) {
        let mut sender = self.get_sender().clone();

        for (_, client) in self.clients.iter() {
            if let Some(except) = except {
                if client.addr == *except {continue}
            }

            messages::send_message(payload.clone(), client.addr.clone(), &mut sender).ok();
        }
    }

    fn handle_hole_punch(&mut self) {
        let mut sender = self.get_sender().clone();
        let session_id = self.session_id.as_ref().unwrap();

        self.clients_to_holepunch.retain_mut(|session| {
        // Send a message every second
            if session.timer.is_some() && session.timer.as_ref().unwrap().elapsed().as_secs() < 1 {return true}

            messages::send_message(Payloads::Handshake {
                session_id: session_id.clone()
            }, session.addr.clone(), &mut sender).ok();
            // Over retry limit, stop connection
            if session.retries > MAX_PUNCH_RETRIES {
                return false
            }
            // Reset second timer
            session.timer = Some(Instant::now());

            return true;
        });
    }

    fn handle_message(&mut self, addr: SocketAddr, payload: Payloads) {
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
                if self.clients.contains_key(name) {
                    messages::send_message(Payloads::InvalidName{}, addr, self.get_sender()).ok();
                    return
                }

                self.clients.insert(name.clone(), Client {
                    addr: addr.clone(),
                    is_observer: false,
                });

                messages::send_message(Payloads::PlayerJoined { name: name.clone(), in_control: false, is_server: false, is_observer: false}, addr, self.get_sender()).ok();

                should_relay = false;
            }
            
            Payloads::TransferControl { from: _, to } => {
                self.in_control = to.clone();
            }
            
            Payloads::Handshake { session_id, ..} => {
                    // Incoming UDP packet from peer
                if let Some(verify_session_id) = self.session_id.as_ref() {
                    if *session_id == *verify_session_id {
                        // TODO: add client
                        messages::send_message(Payloads::PeerEstablished {peer: addr}, self.rendevous_server.as_ref().unwrap().clone(), self.get_sender()).ok();
                    }
                }
                should_relay = false;
            }
            Payloads::HostingReceived { session_id } => {
                self.session_id = Some(session_id.clone());
                should_relay = false;
            }
            Payloads::AttemptConnection { peer } => {
                self.clients_to_holepunch.push(HolePunchSession::new(peer.clone()));
                should_relay = false;
            }
            
        }

        if should_relay {
            self.send_to_all(Some(&addr), payload.clone());
        }

        self.server_tx.send(payload).ok();
    }

    fn handle_app_message(&mut self) {
        if let Ok(payload) = self.client_rx.try_recv() {
            self.send_to_all(None, payload);
        }
    }

    fn should_stop(&self) -> bool {
        self.should_stop.load(SeqCst)
    }
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

pub struct Server {
    number_connections: Arc<AtomicU16>,
    should_stop: Arc<AtomicBool>,
    last_stop_reason: Option<String>,

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
            last_stop_reason: None,
            client_rx, client_tx, server_rx, server_tx,
            transfer: None,
            username: username
        }
    }

    fn port_forward(&self, port: u16) -> Result<(), PortForwardResult> {
        let local_addr = match local_ipaddress::get() {
            Some(addr) => Ipv4Addr::from_str(addr.as_str()).unwrap(),
            None => return Err(PortForwardResult::LocalAddrNotFound)
        };

        let gateway = match search_gateway(igd::SearchOptions {
                bind_addr: SocketAddr::new(IpAddr::V4(local_addr), 0), 
                timeout: Some(Duration::from_secs(3)),
                ..Default::default()}) 
        {
            Ok(g) => g,
            Err(e) => return Err(PortForwardResult::GatewayNotFound(e))
        };

        match gateway.add_port(PortMappingProtocol::UDP, port, SocketAddrV4::new(local_addr, port), 86400, "YourControls") {
            Ok(()) => {},
            Err(e) => return Err(PortForwardResult::AddPortError(e))
        };

        Ok(())
    }

    pub fn start(&mut self, is_ipv6: bool, port: u16) -> Result<(Sender<Payloads>, Receiver<Payloads>), laminar::ErrorKind> {
        let socket = Socket::bind_with_config(get_bind_address(is_ipv6, Some(port)), get_socket_config())?;
        // Attempt to port forward
        if let Err(e) = self.port_forward(port) {
            warn!("Could not port forward! Reason: {:?}", e)
        }

        messages::send_message(Payloads::Handshake {session_id: String::new()}, get_rendezvous_server(is_ipv6).unwrap(), &mut socket.get_packet_sender()).ok();
        
        self.run(socket, None)
    }

    pub fn start_with_hole_punching(&mut self, is_ipv6: bool) -> Result<(Sender<Payloads>, Receiver<Payloads>), laminar::ErrorKind> {
        let socket = Socket::bind_with_config(get_bind_address(is_ipv6, None), get_socket_config())?;
        let addr: SocketAddr = get_rendezvous_server(is_ipv6).unwrap();

        // Send message to external server to obtain session ID
        messages::send_message(Payloads::Handshake {session_id: String::new()}, addr.clone(), &mut socket.get_packet_sender()).ok();

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
            should_stop: self.should_stop.clone()
        }));
        let transfer_thread_clone = transfer.clone();

        self.transfer = Some(transfer);

        // Run socket
        let should_stop_clone = self.should_stop.clone();
        thread::spawn(move || loop {
            socket.manual_poll(Instant::now());
            if should_stop_clone.load(SeqCst) {break}
            sleep(Duration::from_millis(1));
        });

        // Run main loop
        thread::spawn(move || {
            loop {
                let mut transfer = transfer_thread_clone.lock().unwrap();

                match messages::get_next_message(&mut transfer.receiver) {
                    Ok((addr, payload)) => {
                        transfer.handle_message(addr, payload);
                    },
                    Err(e) => match e {
                        Error::SerdeError(_) => {continue;}
                        Error::ConnectionClosed(_) => {continue;}
                        Error::Dummy => {continue;}
                        Error::ReadTimeout => {}
                    }
                };

                transfer.handle_hole_punch();
                transfer.handle_app_message();

                if transfer.should_stop() {break}
            }
        });

        Ok((self.client_tx.clone(), self.server_rx.clone()))
    }
}

impl TransferClient for Server {
    fn get_connected_count(&self) -> u16 {
        return self.number_connections.load(SeqCst);
    }

    fn is_server(&self) -> bool {
        true
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
        }).ok();
    }

    fn get_session_id(&self) -> Option<String> {
        if let Some(transfer) = self.transfer.as_ref() {
            return transfer.lock().unwrap().session_id.clone()
        }
        return None
    }

    fn get_stop_atomic(&self) -> &Arc<AtomicBool> {
        &self.should_stop
    }

    fn get_last_stop_reason(&mut self) -> &mut Option<String> {
        &mut self.last_stop_reason
    }
}