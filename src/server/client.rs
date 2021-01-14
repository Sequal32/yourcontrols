use crossbeam_channel::{Receiver, Sender, unbounded};
use log::info;
use laminar::{Socket};
use spin_sleep::sleep;
use std::{net::{SocketAddr}, net::IpAddr, sync::Mutex, time::Duration, time::Instant, mem};
use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
use std::thread;

use super::{Error, Event, HEARTBEAT_INTERVAL_MANUAL_SECS, LOOP_SLEEP_TIME_MS, MAX_PUNCH_RETRIES, Message, Payloads, ReceiveMessage, SenderReceiver, StartClientError, get_bind_address, get_rendezvous_server, get_socket_config, match_ip_address_to_socket_addr, messages, util::{TransferClient}};

struct TransferStruct {
    name: String,
    version: String,
    // Internally receive data to send to clients
    client_rx: Receiver<Payloads>,
    // Send data to app to receive client data
    server_tx: Sender<ReceiveMessage>,
    // Reading/writing to UDP stream
    net: SenderReceiver,
    // Hole punching
    connected: bool,
    received_address: Option<SocketAddr>,
    retry_timer: Option<Instant>,
    session_id: String,
    retries: u8,
    // State
    should_stop: Arc<AtomicBool>,
    heartbeat_instant: Instant
}

impl TransferStruct {
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(SeqCst)
    }

    // Should stop client
    fn handle_message(&mut self, addr: SocketAddr, payload: Payloads) {
        match &payload {
            // Unused by client
            Payloads::HostingReceived { .. } |
            Payloads::InitHandshake { .. } |
            Payloads::PeerEstablished { .. } |
            Payloads::Ready |
            // No futher handling required
            Payloads::TransferControl { ..} |
            Payloads::SetObserver { .. } |
            Payloads::PlayerJoined { .. } |
            Payloads::PlayerLeft { .. } |
            Payloads::Update { .. } |
            Payloads::Heartbeat => {}
            // Used
            Payloads::InvalidVersion { server_version } => {
                self.stop(format!("Server has mismatching version {}", server_version));
            }
            Payloads::InvalidName { .. } => {
                self.stop(format!("{} already in use!", self.name));
            }
            Payloads::Handshake { session_id } => {
                // Already established connection
                if self.connected {return}
                // Why doesn't the other peer have the same session ID? 
                if *session_id != *self.session_id {
                    self.stop(format!("Handshake verification failed! Expected {}, got {}", self.session_id, session_id));
                    return;
                }
                // Established connection with host
                self.connected = true;

                // Send initial data
                self.net.send_message(Payloads::InitHandshake {
                    name: self.name.clone(),
                    version: self.version.clone(),
                }, addr.clone()).ok();
                
                
                info!("[NETWORK] Established connection with {} on {}!", addr, session_id);

                self.server_tx.try_send(ReceiveMessage::Event(Event::ConnectionEstablished)).ok();
            }
            Payloads::AttemptConnection { peer } => {
                self.received_address = Some(peer.clone())
            }
        }

        self.server_tx.try_send(ReceiveMessage::Payload(payload)).ok();
    }

    fn handle_app_message(&mut self) {
        while let Ok(payload) = self.client_rx.try_recv() {
            if let Some(address) = self.received_address {
                self.net.send_message(payload, address).ok();
            }
        }
    }

    // Returns whether to stop client (can't establish connection)
    fn handle_handshake(&mut self) {
        if self.connected {return}

        // Send a message every second
        if let Some(timer) = self.retry_timer.as_ref() {if timer.elapsed().as_secs() < 1 {return}}

        if let Some(addr) = self.received_address {
            self.net.send_message(Payloads::Handshake {session_id: self.session_id.clone()}, addr).ok();
            // Reset second timer
            self.retry_timer = Some(Instant::now());
            self.retries += 1;

            // Over retry limit, stop connection
            if self.retries == MAX_PUNCH_RETRIES {
                self.should_stop.store(true, SeqCst);
                self.server_tx.try_send(ReceiveMessage::Event(Event::UnablePunchthrough)).ok();
            }

            info!("[NETWORK] Sent packet to {}. Retry #{}", addr, self.retries);
        }
    }

    // Reliably compared to default heartbeat implementation
    fn handle_heartbeat(&mut self) {
        if !self.connected {return}
        if let Some(addr) = self.received_address {
            
            if self.heartbeat_instant.elapsed().as_secs_f32() < HEARTBEAT_INTERVAL_MANUAL_SECS {return}

            self.heartbeat_instant = Instant::now();
            self.net.send_message(Payloads::Heartbeat, addr).ok();

        }
    }

    fn stop(&mut self, reason: String) {
        self.server_tx.try_send(ReceiveMessage::Event(Event::ConnectionLost(reason))).ok();
        self.should_stop.store(true, SeqCst);
    }
}

pub struct Client {
    should_stop: Arc<AtomicBool>,
    transfer: Option<Arc<Mutex<TransferStruct>>>,
    // Recieve data from clients
    server_rx: Receiver<ReceiveMessage>,
    // Send data to clients
    client_tx: Sender<Payloads>,
    // Internally receive data to send to clients
    client_rx: Receiver<Payloads>,
    // Send data to app to receive client data
    server_tx: Sender<ReceiveMessage>,
    // IP
    username: String,
    version: String,
    timeout: u64
}

impl Client {
    pub fn new(username: String, version: String, timeout: u64) -> Self {
        let (client_tx, client_rx) = unbounded();
        let (server_tx, server_rx) = unbounded();

        Self {
            should_stop: Arc::new(AtomicBool::new(false)),
            timeout,
            transfer: None,
            client_rx, client_tx, server_rx, server_tx,
            username,
            version,
        }
    }

    fn get_socket(&self, is_ipv6: bool) -> Result<Socket, laminar::ErrorKind> {
        Socket::bind_with_config(get_bind_address(is_ipv6, None), get_socket_config(self.timeout))
    }

    pub fn start(&mut self, ip: IpAddr, port: u16) -> Result<(), StartClientError> {
        self.run(ip.is_ipv6(), String::new(), None, Some(match_ip_address_to_socket_addr(ip, port)))
    }

    pub fn start_with_hole_punch(&mut self, session_id: String, is_ipv6: bool) -> Result<(), StartClientError> {
        self.run(is_ipv6, session_id, Some(get_rendezvous_server(is_ipv6)?), None)
    }

    pub fn run(&mut self, is_ipv6: bool, session_id: String, rendezvous: Option<SocketAddr>, target_address: Option<SocketAddr>) -> Result<(), StartClientError> {
        let mut socket = self.get_socket(is_ipv6)?;

        info!("[NETWORK] Listening on {:?}", socket.local_addr());

        let mut transfer = TransferStruct {
            // Transfer
            client_rx: self.client_rx.clone(),
            server_tx: self.server_tx.clone(),
            net: SenderReceiver::from_socket(&socket),
            // Holepunching
            retries: 0,
            connected: false,
            received_address: target_address,
            retry_timer: None,
            session_id: session_id.clone(),
            // State
            name: self.get_server_name().to_string(),
            version: self.version.clone(),
            should_stop: self.should_stop.clone(),
            heartbeat_instant: Instant::now()
        };

        if let Some(rendezvous) = rendezvous {
            // Send a handshake to rendezvous to resolve session id with an ip address
            transfer.net.send_message(Payloads::Handshake {
                session_id: session_id.clone()
            }, rendezvous.clone()).ok();
        } else if let Some(addr) = target_address {
            // Send a handshake to the target address to start establishing a connection
            transfer.net.send_message(Payloads::Handshake {
                session_id: String::new(),
            }, addr.clone()).ok();
        }

        let transfer_send = Arc::new(Mutex::new(transfer));
        let transfer_thread_clone = transfer_send.clone();

        self.transfer = Some(transfer_send);

        let rendezvous_timer = Instant::now();
        let timeout = self.timeout;
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
                                // Can't connect to rendezvous to obtain session key
                            if rendezvous.is_none() || (rendezvous.is_some() && rendezvous.unwrap() != addr) {
                                transfer.stop("No message received from server.".to_string())
                            }
                        }
                        Ok(Message::Metrics(addr, metrics)) => {
                            // Send message from game server, not rendezvous
                            if let Some(received_address) = transfer.received_address {
                                if received_address == addr {
                                    transfer.server_tx.send(ReceiveMessage::Event(Event::Metrics(metrics))).ok();
                                }
                            }
                        }
                        Err(e) => match e {
                            Error::ReadTimeout(_) => break,
                            _ => {}
                        }
                    };
                }

                // Check rendezvous timer
                if transfer.received_address.is_none() && rendezvous.is_some() && rendezvous_timer.elapsed().as_secs() >= timeout {
                    transfer.stop("Could not connect to session.".to_string())
                }

                transfer.handle_handshake();
                transfer.handle_app_message();
                transfer.handle_heartbeat();

                if transfer.should_stop() {break}

                mem::drop(transfer);

                sleep(sleep_duration);
            }
        });

        Ok(())
    }
}

impl TransferClient for Client {
    fn get_connected_count(&self) -> u16 {
        return 1;
    }

    fn is_server(&self) -> bool {
        false
    }

    fn get_transmitter(&self) -> &Sender<Payloads> {
        return &self.client_tx
    }

    fn get_server_transmitter(&self) -> &Sender<ReceiveMessage> {
        return &self.server_tx
    }

    fn get_receiver(&self) -> &Receiver<ReceiveMessage> {
        return &self.server_rx
    }

    fn get_server_name(&self) -> &str {
        return &self.username
    }

    fn get_session_id(&self) -> Option<String> {
        if let Some(transfer) = self.transfer.as_ref() {
            return Some(transfer.lock().unwrap().session_id.clone())
        }
        None
    }

    fn stop(&mut self, reason: String) {
        self.should_stop.store(true, SeqCst);
        self.server_tx.try_send(ReceiveMessage::Event(Event::ConnectionLost(reason))).ok();
    }
}