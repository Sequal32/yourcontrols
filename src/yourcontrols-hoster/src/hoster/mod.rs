mod communicator;
mod state;

use communicator::{Communicator, HosterPayloads};
use laminar::{Metrics, Socket};
use state::ActiveState;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread::sleep;
use std::time::{Duration, Instant};

use yourcontrols_net::{get_socket_config, Message, Payloads, SenderReceiver};

const SERVER_TIMEOUT: u64 = 3;

pub struct Hoster {
    net: SenderReceiver,
    communicator: Communicator,
    metrics_data: HashMap<SocketAddr, Metrics>,
    servers: ActiveState,
    cleanup_timer: Instant,
}

impl Hoster {
    pub fn new(port: u16, communicator_address: SocketAddr) -> Self {
        let socket = Socket::bind_with_config(
            SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port),
            get_socket_config(SERVER_TIMEOUT),
        )
        .expect("Failed to bind!");

        Self {
            net: SenderReceiver::from_socket(socket),
            communicator: Communicator::new(communicator_address),
            metrics_data: HashMap::new(),
            servers: ActiveState::new(),
            cleanup_timer: Instant::now(),
        }
    }

    fn handle_heartbeats(&mut self) {
        for (_, state) in self.servers.get_server_states().iter_mut() {
            if state.heartbeat_instant.elapsed().as_secs_f32() < 0.5 {
                return;
            }

            state.send_to_all(Payloads::Heartbeat, None, &mut self.net);

            state.heartbeat_instant = Instant::now();
        }
    }

    fn process_net(&mut self) {
        self.net.poll();

        while let Ok(message) = self.net.get_next_message() {
            match message {
                Message::Payload(addr, payload) => {
                    // Get server state for session
                    if let Some(state) = self.servers.get_server_state_for(&addr) {
                        state.process_payload(addr, payload, &mut self.net);
                    }
                }
                Message::ConnectionClosed(addr) => {
                    let session = match self.servers.remove_client(&addr) {
                        Some(s) => s,
                        None => continue,
                    };

                    self.metrics_data.remove(&addr);

                    let mut should_close = false;

                    let state = match self.servers.get_server_state(&session) {
                        Some(s) => s,
                        None => continue,
                    };

                    if let Some(name) = state.remove_client_by_addr(&addr) {
                        // If was in control... need to transfer it to someone else if they're still in
                        if !state.clients.is_empty() {
                            if name == state.in_control {
                                let next = state.clients.iter().next().unwrap().0.clone();
                                state.set_host(next, &mut self.net);
                            }
                        } else {
                            // Close server
                            should_close = true;
                        }

                        state.send_to_all(Payloads::PlayerLeft { name }, None, &mut self.net);
                    }

                    if should_close {
                        self.servers.remove_server(&session);
                    }
                }
                Message::Metrics(addr, metrics) => {
                    self.metrics_data.insert(addr, metrics);
                }
            }
        }
    }

    fn process_communicator(&mut self) {
        match self.communicator.poll() {
            Ok(HosterPayloads::HostingRequested { session_id }) => {
                self.servers.add_server(session_id.clone());

                self.communicator
                    .send_message(HosterPayloads::SessionOpen { session_id });
            }
            Ok(HosterPayloads::ClientConnecting { session_id, addr }) => {
                self.servers.add_client(addr, session_id);

                self.communicator
                    .send_message(HosterPayloads::ClientAcknowledged { addr });
            }
            _ => {}
        }
    }

    fn cleanup(&mut self) {
        if self.cleanup_timer.elapsed().as_secs() < 10 {
            return;
        }

        for removed in self.servers.remove_unused() {
            self.communicator
                .send_message(HosterPayloads::SessionClosed {
                    session_id: removed,
                });
        }

        self.cleanup_timer = Instant::now();
    }

    pub fn run(&mut self) {
        loop {
            self.process_net();
            self.process_communicator();
            self.handle_heartbeats();
            self.cleanup();

            sleep(Duration::from_millis(1));
        }
    }
}
