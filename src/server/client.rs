use crossbeam_channel::{Receiver, Sender, unbounded};

use laminar::{Packet, Socket, SocketEvent};
use std::{time::Instant, net::{SocketAddr}, sync::Mutex};
use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
use std::thread;

use super::{Error, MAX_PUNCH_RETRIES, Payloads, get_bind_address, get_rendezvous_server, messages, util::{TransferClient}};

struct TransferStruct {
    name: String,
    // Internally receive data to send to clients
    client_rx: Receiver<Payloads>,
    // Send data to app to receive client data
    server_tx: Sender<Payloads>,
    // Reading/writing to UDP stream
    receiver: Receiver<SocketEvent>,
    sender: Sender<Packet>,
    // Hole punching
    connected: bool,
    server_address: SocketAddr,
    received_address: Option<SocketAddr>,
    retry_timer: Option<Instant>,
    session_id: String,
    retries: u8
}

impl TransferStruct {
    pub fn get_sender(&mut self) -> &mut Sender<Packet> {
        &mut self.sender
    }
}

// Should stop client
fn handle_message(addr: SocketAddr, payload: Payloads, transfer: &mut TransferStruct) -> bool {
    match &payload {
        // Unused by client
        Payloads::HostingReceived { .. } => {}
        Payloads::Name { .. } => {}
        Payloads::PeerEstablished { .. } => {}
        // No futher handling required
        Payloads::TransferControl { ..} => {}
        Payloads::SetObserver { .. } => {}
        Payloads::InvalidName { .. } => {}
        Payloads::PlayerJoined { .. } => {}
        Payloads::PlayerLeft { .. } => {}
        Payloads::Update { .. } => {}
        // Used
        Payloads::Handshake { is_initial, session_id } => {
            if *session_id != transfer.session_id {return false}
            // Established connection with server
            transfer.connected = true;

            messages::send_message(Payloads::Name {name: transfer.name.clone()}, addr, transfer.get_sender());
        }
        Payloads::AttemptConnection { peer } => {
            transfer.received_address = Some(peer.clone())
        }
        
    }

    transfer.server_tx.send(payload);
    return true
}

fn handle_app_message(transfer: &mut TransferStruct) {
    if let Ok(payload) = transfer.client_rx.try_recv() {
        messages::send_message(payload, transfer.received_address.unwrap(), transfer.get_sender());
    }
}

// Returns whether to stop client (can't establish connection)
fn handle_hole_punch(transfer: &mut TransferStruct) -> bool {
    let sender = &mut transfer.sender;

    // Send a message every second
    if transfer.retry_timer.is_some() && transfer.retry_timer.as_ref().unwrap().elapsed().as_secs() < 1 {return false}

    messages::send_message(Payloads::Handshake {is_initial: true, session_id: transfer.session_id.clone()}, transfer.server_address.clone(), sender);
    // Over retry limit, stop connection
    if transfer.retries > MAX_PUNCH_RETRIES {
        return false
    }
    // Reset second timer
    transfer.retry_timer = Some(Instant::now());

    return true
}

pub struct Client {
    should_stop: Arc<AtomicBool>,
    transfer: Option<Arc<Mutex<TransferStruct>>>,
    // Recieve data from clients
    server_rx: Receiver<Payloads>,
    // Send data to clients
    client_tx: Sender<Payloads>,
    // Internally receive data to send to clients
    client_rx: Receiver<Payloads>,
    // Send data to app to receive client data
    server_tx: Sender<Payloads>,
    // IP
    username: String
}

impl Client {
    pub fn new(username: String) -> Self {
        let (client_tx, client_rx) = unbounded();
        let (server_tx, server_rx) = unbounded();

        Self {
            should_stop: Arc::new(AtomicBool::new(false)),
            transfer: None,
            client_rx, client_tx, server_rx, server_tx,
            username: username
        }
    }

    pub fn start(&mut self, session_id: String, is_ipv6: bool) -> Result<(), laminar::ErrorKind> {
        let mut socket = Socket::bind(get_bind_address(is_ipv6, None))?;

        let transfer = Arc::new(Mutex::new(
            TransferStruct {
                // Transfer
                client_rx: self.client_rx.clone(),
                server_tx: self.server_tx.clone(),
                receiver: socket.get_event_receiver(), 
                sender: socket.get_packet_sender(),
                // Holepunching
                retries: 0,
                connected: false,
                server_address: get_rendezvous_server(is_ipv6),
                received_address: None,
                retry_timer: None,
                session_id: session_id.clone(),
                // State
                name: self.get_server_name().to_string(),
            }
        ));
        let transfer_thread_clone = transfer.clone();

        self.transfer = Some(transfer);

        // Init connection
        messages::send_message(Payloads::Handshake {
            is_initial: false,
            session_id,
        }, get_rendezvous_server(is_ipv6), &mut socket.get_packet_sender());

        thread::spawn(move || socket.start_polling());
        thread::spawn(move || {
            loop {
                let mut transfer = &mut transfer_thread_clone.lock().unwrap();
                
                let (addr, payload) = match messages::get_next_message(&mut transfer.receiver) {
                    Ok(a) => a,
                    Err(e) => match e {
                        Error::SerdeError(_) => {continue}
                        Error::ConnectionClosed(_) => {continue}
                        Error::Dummy => {continue}
                    }
                };

                if !transfer.connected {handle_hole_punch(&mut transfer);};
                handle_message(addr, payload, transfer);
                handle_app_message(transfer);
            }
        });

        Ok(())
    }
}

impl TransferClient for Client {
    fn get_connected_count(&self) -> u16 {
        return 1;
    }

    fn stop(&self, reason: String) {
        self.should_stop.store(true, SeqCst);
    }

    fn stopped(&self) -> bool {
        self.should_stop.load(SeqCst)
    }

    fn is_server(&self) -> bool {
        false
    }

    fn get_transmitter(&self) -> &Sender<Payloads> {
        return &self.client_tx
    }

    fn get_receiver(&self) -> &Receiver<Payloads> {
        return &self.server_rx
    }

    fn get_server_name(&self) -> &str {
        return &self.username
    }

    fn get_session_id(&self) -> Option<String> {
        return None
    }
}