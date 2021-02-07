use crossbeam_channel::{Receiver, Sender, unbounded};
use log::{error, info};
use igd::{PortMappingProtocol, SearchOptions, search_gateway};
use local_ipaddress;
use messages::Message;
use retain_mut::RetainMut;
use spin_sleep::sleep;
use std::{collections::HashMap, mem, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}, thread, time::Duration, time::Instant};
use laminar::{Metrics, Socket};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU16, Ordering::SeqCst}};
use super::{ClientReceiver, ClientSender, Error, Event, HEARTBEAT_INTERVAL_MANUAL_SECS, LOOP_SLEEP_TIME_MS, MAX_PUNCH_RETRIES, Payloads, PortForwardResult, ReceiveMessage, SenderReceiver, ServerReceiver, ServerSender, StartClientError, get_bind_address, get_rendezvous_server, get_socket_config, messages, util::{TransferClient}};

struct Client {
    addr: SocketAddr,
    is_observer: bool
}

struct TransferStruct {
    session_id: String,
    clients: HashMap<String, Client>,
    // Reading/writing to UDP stream
    net: SenderReceiver,
    // Holepunching
    rendezvous_server: Option<SocketAddr>,
    clients_to_holepunch: Vec<HolePunchSession>,
    // Sending/writing to app
    server_tx: ServerSender,
    client_rx: ClientReceiver,
    // State
    in_control: String,
    should_stop: Arc<AtomicBool>,
    number_connections: Arc<AtomicU16>,
    username: String,
    version: String,
    heartbeat_instant: Instant,
    // Metrics
    metrics: HashMap<SocketAddr, Metrics>,
    metrics_instant: Instant
}

impl TransferStruct {
    fn send_to_all(&mut self, except: Option<&SocketAddr>, payload: Payloads) {
        let mut to_send = Vec::new();

        for (_, client) in self.clients.iter() {
            if let Some(except) = except {
                if client.addr == *except {continue}
            }

            to_send.push(client.addr.clone());
        }

        self.net.send_message_to_multiple(payload, to_send).ok();
    }

    fn handle_handshake(&mut self) {
        if self.clients_to_holepunch.len() == 0 {return;}

        let session_id = self.session_id.clone();
        let mut to_send = Vec::new();

        self.clients_to_holepunch.retain_mut(|session| {
        // Send a message every second
            if let Some(timer) = session.timer.as_ref() {if timer.elapsed().as_secs() < 1 {return true;}}            
        
            to_send.push((Payloads::Handshake {
                session_id: session_id.clone()
            }, session.addr.clone()));
            // Reset second timer
            session.retries += 1;
            session.timer = Some(Instant::now());

            // Over retry limit, stop connection
            if session.retries == MAX_PUNCH_RETRIES {
                return false
            }

            info!("[NETWORK] Sent handshake packet to {}. Retry #{}", session.addr, session.retries);

            return true;
        });

        for (payload, addr) in to_send {
            self.net.send_message(payload, addr).ok();
        }
    }

    fn handle_message(&mut self, addr: SocketAddr, payload: Payloads) {
        let mut should_relay = true;

        match &payload {
            // Unused for server
            Payloads::InvalidName { .. } |
            Payloads::InvalidVersion { .. } |
            Payloads::PlayerJoined { .. } |
            Payloads::PlayerLeft { .. } |
            Payloads::SetObserver { .. } |
            Payloads::RequestHosting { .. } |
            Payloads::AircraftDefinition { .. } |
            Payloads::Heartbeat |
            Payloads::SetHost |
            Payloads::PeerEstablished { .. } => {return}  // No client should be able to send this
            // No processing needed
            Payloads::Update { .. } => {}
            Payloads::Ready => {}
            // Used
            Payloads::InitHandshake { name, version } => {
                // Version check
                if *version != self.version {
                    self.net.send_message(Payloads::InvalidVersion {
                        server_version: self.version.clone(),
                    }, addr).ok();
                    return;
                }

                info!("[NETWORK] Client requests name {}", name);
                // Name already in use by another client
                let mut invalid_name = *name == self.username;
                // Lookup name if it exists already
                if let Some(client) = self.clients.get(name) {
                    // Same client might've send packet twice
                    if client.addr == addr {return}
                    invalid_name = true;
                }

                if invalid_name {
                    self.net.send_message(Payloads::InvalidName{}, addr).ok();
                    return;
                }

                // Send all connected clients to new player
                for (name, client) in self.clients.iter() {
                    self.net.send_message(Payloads::PlayerJoined { 
                        name: name.clone(), 
                        in_control: self.in_control == *name, 
                        is_server: false, 
                        is_observer: client.is_observer
                    }, addr).ok();
                }
                // Send self
                self.net.send_message(Payloads::PlayerJoined { name: self.username.clone(), in_control: self.in_control == self.username, is_server: true, is_observer: false}, addr).ok();
                // Add client
                self.clients.insert(name.clone(), Client {addr: addr.clone(),
                    is_observer: false,
                });

                self.number_connections.fetch_add(1, SeqCst);

                let empty_new_player = Payloads::PlayerJoined { name: name.clone(), in_control: false, is_server: false, is_observer: false};

                self.send_to_all(Some(&addr), empty_new_player.clone());
                self.server_tx.try_send(ReceiveMessage::Payload(empty_new_player)).ok();
                // Early return to prevent relaying/sending payload
                return
            }
            
            Payloads::TransferControl { from: _, to } => {
                self.in_control = to.clone();
            }
            
            Payloads::Handshake { session_id, ..} => {
                info!("[NETWORK] Handshake received from {} on {}", addr, session_id);
                    // Incoming UDP packet from peer
                if *session_id == self.session_id {
                    self.net.send_message(Payloads::Handshake{session_id: session_id.clone()}, addr).ok();
                    
                    if let Some(rendezvous) = self.rendezvous_server.as_ref() {
                        self.net.send_message(Payloads::PeerEstablished {peer: addr}, *rendezvous).ok();

                        self.clients_to_holepunch.retain(|x| {
                            x.addr != addr
                        });
                    }
                }

                should_relay = false;
            }
            Payloads::HostingReceived { session_id } => {
                info!("[NETWORK] Obtained session ID: {}", session_id);
                self.session_id = session_id.clone();
                should_relay = false;

                self.server_tx.try_send(ReceiveMessage::Event(Event::ConnectionEstablished)).ok();
            }
            Payloads::AttemptConnection { peer } => {
                info!("[NETWORK] {} attempting connection.", peer);
                self.clients_to_holepunch.push(HolePunchSession::new(peer.clone()));
                should_relay = false;
            }
            
        }

        if should_relay {
            self.send_to_all(Some(&addr), payload.clone());
        }

        self.server_tx.try_send(ReceiveMessage::Payload(payload)).ok();
    }

    fn handle_app_message(&mut self) {
        while let Ok((payload, target)) = self.client_rx.try_recv() {
            match &payload {
                Payloads::TransferControl { from: _, to } => {
                    self.in_control = to.clone();
                }
                _ => {}
            }

            if let Some(target) = target {
                if let Some(client) = self.clients.get(&target) {
                    self.net.send_message(payload, client.addr).ok();
                }
            } else {
                self.send_to_all(None, payload);
            }
        }
    }

    // Reliably compared to default heartbeat implementation
    fn handle_heartbeat(&mut self) {
        if self.heartbeat_instant.elapsed().as_secs_f32() < HEARTBEAT_INTERVAL_MANUAL_SECS {return}

        self.heartbeat_instant = Instant::now();
        self.send_to_all(None, Payloads::Heartbeat);
    }

    fn handle_metrics(&mut self) {
        if self.metrics_instant.elapsed().as_secs_f32() < 1.0 {return}

        let mut all_metrics = Metrics::default();

        for metric in self.metrics.values().cloned() {
            all_metrics += metric;
        }

        self.metrics_instant = Instant::now();
        
        self.server_tx.send(ReceiveMessage::Event(Event::Metrics(all_metrics))).ok();
    }

    fn remove_client(&mut self, addr: SocketAddr) {
        let mut removed_client_name: Option<String> = None;

        self.clients.retain(|name, client| {
            if client.addr == addr {
                removed_client_name = Some(name.clone());
                return false
            }
            return true
        });

        info!("[NETWORK] Removing client from port {} who has name {:?}", addr.port(), removed_client_name);

        if let Some(name) = removed_client_name {
            let player_left_payload = Payloads::PlayerLeft {name: name.clone()};

            self.send_to_all(None, player_left_payload.clone());
            self.number_connections.fetch_sub(1, SeqCst);
            self.server_tx.try_send(ReceiveMessage::Payload(player_left_payload)).ok();
        }

        self.metrics.remove(&addr);
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

    transfer: Option<Arc<Mutex<TransferStruct>>>,

    last_port_forward_result: Option<Result<(), PortForwardResult>>,
    // Send data to peers
    client_tx: ClientSender,
    // Internally receive data to send to clients
    client_rx: ClientReceiver,

    // Send data to app to receive client data
    server_tx: ServerSender,
    // Recieve data from server
    server_rx: ServerReceiver,

    username: String,
    version: String,
    timeout: u64
}

impl Server {
    pub fn new(username: String, version: String, timeout: u64) -> Self  {
        let (client_tx, client_rx) = unbounded();
        let (server_tx, server_rx) = unbounded();

        return Self {
            number_connections: Arc::new(AtomicU16::new(0)),
            
            last_port_forward_result: None,
            should_stop: Arc::new(AtomicBool::new(false)),
            client_rx, client_tx, server_rx, server_tx,
            transfer: None,
            username, version, timeout
        }
    }

    fn port_forward(&self, port: u16) -> Result<(), PortForwardResult> {
        let local_addr = match local_ipaddress::get() {
            Some(addr) => match addr.parse::<Ipv4Addr>() {
                Ok(addr) => addr,
                Err(_) => return Err(PortForwardResult::LocalAddrNotIPv4(addr))
            },
            None => return Err(PortForwardResult::LocalAddrNotFound)
        };

        info!("[NETWORK] Found local address: {}", local_addr);

        let gateway = match search_gateway(SearchOptions {
                bind_addr: SocketAddr::new(IpAddr::V4(local_addr), 0), 
                timeout: Some(Duration::from_secs(3)),
                ..Default::default()}) 
        {
            Ok(g) => g,
            Err(e) => return Err(PortForwardResult::GatewayNotFound(e))
        };

        info!("[NETWORK] Found gateway at {}", gateway.root_url);

        match gateway.add_port(PortMappingProtocol::UDP, port, SocketAddrV4::new(local_addr, port), 86400, "YourControls") {
            Ok(()) => {},
            Err(e) => return Err(PortForwardResult::AddPortError(e))
        };

        info!("[NETWORK] Port forwarded port {}", port);

        Ok(())
    }

    pub fn start(&mut self, is_ipv6: bool, port: u16, upnp: bool) -> Result<(), StartClientError> {
        let socket = Socket::bind_with_config(get_bind_address(is_ipv6, Some(port)), get_socket_config(self.timeout))?;
        // Attempt to port forward
        if upnp {
            self.last_port_forward_result = Some(self.port_forward(port));
        }
        
        self.run(socket, None)
    }

    pub fn start_with_hole_punching(&mut self, is_ipv6: bool) -> Result<(), StartClientError> {
        let socket = Socket::bind_with_config(get_bind_address(is_ipv6, None), get_socket_config(self.timeout))?;
        let addr: SocketAddr = get_rendezvous_server(is_ipv6)?;

        self.run(socket, Some(addr))
    }

    fn run(&mut self, socket: Socket, rendezvous: Option<SocketAddr>) -> Result<(), StartClientError> {
        let mut socket = socket;

        info!("[NETWORK] Listening on {:?}", socket.local_addr());

        let mut transfer = TransferStruct {
            // Holepunching
            session_id: String::new(),
            rendezvous_server: rendezvous,
            clients_to_holepunch: Vec::new(),
            // Transfer
            server_tx: self.server_tx.clone(),
            client_rx: self.client_rx.clone(),
            net: SenderReceiver::from_socket(&socket),
            // State
            in_control: self.username.clone(),
            clients: HashMap::new(),
            should_stop: self.should_stop.clone(),
            number_connections: self.number_connections.clone(),
            username: self.username.clone(),
            version: self.version.clone(),
            heartbeat_instant: Instant::now(),
            metrics_instant: Instant::now(),
            metrics: HashMap::new()
        };
        

        if let Some(addr) = rendezvous {
            // Send handshake payload to rendezvous server to get session ID
            transfer.net.send_message(Payloads::RequestHosting {self_hosted: true}, addr).ok();
        } else {
            // If not hole punching, then tell the application that the server is immediately ready
            self.server_tx.send(ReceiveMessage::Event(Event::ConnectionEstablished)).ok();
        }

        let transfer_send = Arc::new(Mutex::new(transfer));
        let transfer_thread_clone = transfer_send.clone();
        self.transfer = Some(transfer_send);

        // Run main loop
        thread::spawn(move || {
            let sleep_duration = Duration::from_millis(LOOP_SLEEP_TIME_MS);
            loop {
                let mut transfer = transfer_thread_clone.lock().unwrap();

                socket.manual_poll(Instant::now());

                loop {
                    match transfer.net.get_next_message() {
                        Ok(Message::Payload(addr, payload)) => {
                            transfer.handle_message(addr, payload);
                        },
                        Ok(Message::ConnectionClosed(addr)) => {
                                // Could not reach rendezvous
                            if transfer.session_id == "" && rendezvous.is_some() && rendezvous.unwrap() == addr {

                                transfer.server_tx.try_send(ReceiveMessage::Event(Event::SessionIdFetchFailed)).ok();
                                transfer.should_stop.store(true, SeqCst);

                            } else {
                                    // Client disconnected
                                transfer.remove_client(addr);
                            }
                        }
                        Ok(Message::Metrics(addr, metrics)) => {
                            transfer.metrics.insert(addr, metrics);
                        }
                        Err(e) => match e {
                            Error::ReadTimeout(_) => break,
                            _ => {}
                        }
                    };
                }

                transfer.handle_handshake();
                transfer.handle_app_message();
                transfer.handle_heartbeat();
                transfer.handle_metrics();

                if transfer.should_stop() {break}

                mem::drop(transfer);
                
                sleep(sleep_duration);
            }
        });

        Ok(())
    }
}

impl TransferClient for Server {
    fn is_host(&self) -> bool {
        true
    }

    fn get_transmitter(&self) -> &ClientSender {
        return &self.client_tx;
    }

    fn get_server_transmitter(&self) -> &ServerSender {
        return &self.server_tx
    }

    fn get_receiver(&self) -> &ServerReceiver {
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
        
        let message = Payloads::TransferControl {
            from: self.get_server_name().to_string(),
            to: target
        };
        self.get_transmitter().try_send((message.clone(), None)).ok();
        self.get_server_transmitter().try_send(ReceiveMessage::Payload(message)).ok();
    }

    fn set_observer(&self, target: String, is_observer: bool) {
        // Read for initial contact with other clients
        if let Some(transfer) = self.transfer.as_ref() {
            if let Some(client) = transfer.lock().unwrap().clients.get_mut(&target) {
                client.is_observer = is_observer;
            }
        }

        self.client_tx.try_send((Payloads::SetObserver {
            from: self.get_server_name().to_string(),
            to: target,
            is_observer: is_observer
        }, None)).ok();
    }

    fn get_session_id(&self) -> Option<String> {
        if let Some(transfer) = self.transfer.as_ref() {
            return Some(transfer.lock().unwrap().session_id.clone())
        }
        return None
    }

    fn stop(&mut self, reason: String) {
        self.should_stop.store(true, SeqCst);
        self.server_tx.try_send(ReceiveMessage::Event(Event::ConnectionLost(reason))).ok();
    }
}