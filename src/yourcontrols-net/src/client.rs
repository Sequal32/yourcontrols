use crossbeam_channel::unbounded;
use laminar::Socket;
use log::info;
use mem::drop;
use spin_sleep::sleep;
use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Arc,
};
use std::thread;
use std::{mem, net::IpAddr, net::SocketAddr, sync::Mutex, time::Duration, time::Instant};

use crate::util::{
    get_bind_address, get_rendezvous_server, get_socket_config, match_ip_address_to_socket_addr,
};
use crate::util::{
    ClientReceiver, ClientSender, Event, ReceiveMessage, ServerReceiver, ServerSender,
    TransferClient,
};
use crate::util::{HEARTBEAT_INTERVAL_MANUAL_SECS, LOOP_SLEEP_TIME_MS, MAX_PUNCH_RETRIES};
use crate::{
    messages::{Message, Payloads, SenderReceiver},
    util::get_local_endpoints_with_port,
};

use yourcontrols_types::Error;

struct TransferStruct {
    name: String,
    version: String,
    // Internally receive data to send to clients
    client_rx: ClientReceiver,
    // Send data to app to receive client data
    server_tx: ServerSender,
    // Reading/writing to UDP stream
    net: SenderReceiver,
    // Hole punching
    received_address: Vec<SocketAddr>,
    connected_address: Option<SocketAddr>,
    retry_timer: Option<Instant>,
    session_id: String,
    retries: u8,
    // State
    should_stop: Arc<AtomicBool>,
    heartbeat_instant: Instant,
}

impl TransferStruct {
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(SeqCst)
    }

    // Should stop client
    fn handle_message(&mut self, addr: SocketAddr, payload: Payloads) {
        match &payload {
            // Unused by client
            Payloads::InitHandshake { .. } |
            Payloads::RendezvousHandshake  { .. } |
            Payloads::PeerEstablished { .. } |
            Payloads::RequestHosting {..} |
            Payloads::Ready |
            Payloads::SetSelfObserver { .. }|
            // No futher handling required
            Payloads::AircraftDefinition { .. } |
            Payloads::TransferControl { ..} |
            Payloads::SetObserver { .. } |
            Payloads::PlayerJoined { .. } |
            Payloads::PlayerLeft { .. } |
            Payloads::Update { .. } |
            Payloads::ConnectionDenied { .. } |
            Payloads::SetHost |
            Payloads::AttemptHosterConnection {..} |
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
                if self.connected() {return}
                // Why doesn't the other peer have the same session ID? 
                if *session_id != *self.session_id {
                    self.stop(format!("Handshake verification failed! Expected {}, got {}", self.session_id, session_id));
                    return;
                }
                // Established connection with host
                self.connected_address = Some(addr);
                self.received_address.drain(..);

                // Send initial data
                self.net.send_message(Payloads::InitHandshake {
                    name: self.name.clone(),
                    version: self.version.clone(),
                }, addr).ok();

                info!("[NETWORK] Established connection with port {} on {}!", addr.port(), session_id);

                self.server_tx.try_send(ReceiveMessage::Event(Event::ConnectionEstablished)).ok();
            }
            Payloads::HostingReceived { session_id } => {
                self.session_id = session_id.clone();
            }
            Payloads::AttemptConnection { peers } => {
                self.received_address = peers.clone();
            }
        }

        self.server_tx
            .try_send(ReceiveMessage::Payload(payload))
            .ok();
    }

    fn handle_app_message(&mut self) {
        while let Ok((payload, _)) = self.client_rx.try_recv() {
            if let Some(address) = self.connected_address {
                self.net.send_message(payload, address).ok();
            }
        }
    }

    // Returns whether to stop client (can't establish connection)
    fn handle_handshake(&mut self) {
        if self.connected() {
            return;
        }

        // Send a message every second
        if let Some(timer) = self.retry_timer.as_ref() {
            if timer.elapsed().as_secs() < 1 {
                return;
            }
        }

        for addr in &self.received_address {
            self.net
                .send_message(
                    Payloads::Handshake {
                        session_id: self.session_id.clone(),
                    },
                    *addr,
                )
                .ok();
            // Reset second timer
            self.retry_timer = Some(Instant::now());
            self.retries += 1;

            // Over retry limit, stop connection
            if self.retries == MAX_PUNCH_RETRIES {
                self.should_stop.store(true, SeqCst);
                self.server_tx
                    .try_send(ReceiveMessage::Event(Event::UnablePunchthrough))
                    .ok();
            }

            info!(
                "[NETWORK] Sent packet to port {}. Retry #{}",
                addr.port(),
                self.retries
            );
        }
    }

    // Reliably compared to default heartbeat implementation
    fn handle_heartbeat(&mut self) {
        if !self.connected() {
            return;
        }

        if let Some(addr) = self.connected_address {
            if self.heartbeat_instant.elapsed().as_secs_f32() < HEARTBEAT_INTERVAL_MANUAL_SECS {
                return;
            }

            self.heartbeat_instant = Instant::now();
            self.net.send_message(Payloads::Heartbeat, addr).ok();
        }
    }

    fn stop(&mut self, reason: String) {
        self.server_tx
            .try_send(ReceiveMessage::Event(Event::ConnectionLost(reason)))
            .ok();
        self.should_stop.store(true, SeqCst);
    }

    fn connected(&self) -> bool {
        self.connected_address.is_some()
    }
}

pub struct Client {
    should_stop: Arc<AtomicBool>,
    transfer: Option<Arc<Mutex<TransferStruct>>>,
    // Recieve data from clients
    server_rx: ServerReceiver,
    // Send data to clients
    client_tx: ClientSender,
    // Internally receive data to send to clients
    client_rx: ClientReceiver,
    // Send data to app to receive client data
    server_tx: ServerSender,
    // IP
    username: String,
    version: String,
    timeout: u64,
    is_host: bool,
}

impl Client {
    pub fn new(username: String, version: String, timeout: u64) -> Self {
        let (client_tx, client_rx) = unbounded();
        let (server_tx, server_rx) = unbounded();

        Self {
            should_stop: Arc::new(AtomicBool::new(false)),
            timeout,
            transfer: None,
            client_rx,
            client_tx,
            server_rx,
            server_tx,
            username,
            version,
            is_host: false,
        }
    }

    fn get_socket(&self, is_ipv6: bool) -> Result<Socket, laminar::ErrorKind> {
        Socket::bind_with_config(
            get_bind_address(is_ipv6, None),
            get_socket_config(self.timeout),
        )
    }

    pub fn start(
        &mut self,
        ip: IpAddr,
        port: u16,
        session_id: Option<String>, // Only used when connecting to the hoster as a secret password
    ) -> Result<(), Error> {
        self.run(
            ip.is_ipv6(),
            session_id,
            None,
            Some(match_ip_address_to_socket_addr(ip, port)),
        )
    }

    pub fn start_with_hole_punch(
        &mut self,
        session_id: String,
        is_ipv6: bool,
    ) -> Result<(), Error> {
        self.run(
            is_ipv6,
            Some(session_id),
            Some(get_rendezvous_server(is_ipv6)?),
            None,
        )
    }

    pub fn start_with_relay(&mut self, is_ipv6: bool) -> Result<(), Error> {
        self.run(is_ipv6, None, Some(get_rendezvous_server(is_ipv6)?), None)
    }

    pub fn run(
        &mut self,
        is_ipv6: bool,
        session_id: Option<String>,
        rendezvous: Option<SocketAddr>,
        target_address: Option<SocketAddr>,
    ) -> Result<(), Error> {
        let socket = self.get_socket(is_ipv6)?;
        let port = socket.local_addr().unwrap().port();

        self.is_host = session_id.is_none() && target_address.is_none();

        info!("[NETWORK] Listening on {:?}", socket.local_addr());

        let mut transfer = TransferStruct {
            // Transfer
            client_rx: self.client_rx.clone(),
            server_tx: self.server_tx.clone(),
            net: SenderReceiver::from_socket(socket),
            // Holepunching
            retries: 0,
            received_address: target_address.map(|x| vec![x]).unwrap_or_default(),
            connected_address: None,
            retry_timer: None,
            session_id: session_id.clone().unwrap_or_default(),
            // State
            name: self.get_server_name().to_string(),
            version: self.version.clone(),
            should_stop: self.should_stop.clone(),
            heartbeat_instant: Instant::now(),
        };

        if let Some(rendezvous) = rendezvous {
            if let Some(session_id) = session_id {
                // Send a handshake to rendezvous to resolve session id with an ip address
                transfer
                    .net
                    .send_message(
                        Payloads::RendezvousHandshake {
                            session_id,
                            local_endpoint: get_local_endpoints_with_port(is_ipv6, port),
                        },
                        rendezvous,
                    )
                    .ok();
            } else {
                transfer
                    .net
                    .send_message(
                        Payloads::RequestHosting {
                            self_hosted: false,
                            local_endpoint: get_local_endpoints_with_port(is_ipv6, port),
                        },
                        rendezvous,
                    )
                    .ok();
            }
        } else if let Some(addr) = target_address {
            info!("Sending request to port {} to join session", addr.port());
            // Send a handshake to the target address to start establishing a connection
            transfer
                .net
                .send_message(
                    Payloads::Handshake {
                        session_id: String::new(),
                    },
                    addr,
                )
                .ok();
        }

        let transfer_send = Arc::new(Mutex::new(transfer));
        let transfer_thread_clone = transfer_send.clone();

        self.transfer = Some(transfer_send);

        let rendezvous_timer = Instant::now();
        // Run main loop
        thread::spawn(move || {
            let sleep_duration = Duration::from_millis(LOOP_SLEEP_TIME_MS);

            loop {
                let mut transfer = transfer_thread_clone.lock().unwrap();

                transfer.net.poll();

                while let Ok(message) = transfer.net.get_next_message() {
                    match message {
                        Message::Payload(addr, payload) => {
                            transfer.handle_message(addr, payload);
                        }
                        Message::ConnectionClosed(addr) => {
                            let was_connected_addr = transfer
                                .connected_address
                                .map(|x| x == addr)
                                .unwrap_or(false);
                            if was_connected_addr {
                                transfer.stop("Connection timeout".to_string())
                            }
                        }
                        Message::Metrics(addr, metrics) => {
                            if let Some(connected_address) = transfer.connected_address {
                                // Record message from game server only, not rendezvous
                                if connected_address == addr {
                                    transfer
                                        .server_tx
                                        .send(ReceiveMessage::Event(Event::Metrics(metrics)))
                                        .ok();
                                }
                            }
                        }
                    }
                }

                // Check rendezvous timer
                if !transfer.connected()
                    && rendezvous.is_some()
                    && rendezvous_timer.elapsed().as_secs() >= 5
                {
                    transfer.stop("Could not connect to session.".to_string())
                }

                transfer.handle_handshake();
                transfer.handle_app_message();
                transfer.handle_heartbeat();

                if transfer.should_stop() {
                    break;
                }

                drop(transfer);
                sleep(sleep_duration);
            }
        });

        Ok(())
    }
}

impl TransferClient for Client {
    fn is_host(&self) -> bool {
        self.is_host
    }

    fn get_transmitter(&self) -> &ClientSender {
        &self.client_tx
    }

    fn get_server_transmitter(&self) -> &ServerSender {
        &self.server_tx
    }

    fn get_receiver(&self) -> &ServerReceiver {
        &self.server_rx
    }

    fn get_server_name(&self) -> &str {
        &self.username
    }

    fn get_session_id(&self) -> Option<String> {
        if let Some(transfer) = self.transfer.as_ref() {
            return Some(transfer.lock().unwrap().session_id.clone());
        }
        None
    }

    fn stop(&mut self, reason: String) {
        self.should_stop.store(true, SeqCst);
        self.server_tx
            .try_send(ReceiveMessage::Event(Event::ConnectionLost(reason)))
            .ok();
    }
}
