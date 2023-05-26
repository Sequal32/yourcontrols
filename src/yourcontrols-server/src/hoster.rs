use crate::servers::{Client, ServerState, Servers};
use laminar::Socket;
use log::info;
use semver::Version;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use yourcontrols_net::{get_socket_config, get_socket_duplex, Message, Payloads, SenderReceiver};
use yourcontrols_types::Error;

pub const SERVER_NAME: &str = "SERVER";

const CLEANUP_INTERVAL: u64 = 30;
const INACTIVE_SESSION_TIMEOUT: u64 = 30;

fn send_to_all(
    payload: Payloads,
    except: Option<&SocketAddr>,
    state: &mut ServerState,
    net: &mut SenderReceiver,
) {
    let mut to_send = Vec::new();

    for (_, client) in state.clients.iter() {
        if let Some(except) = except {
            if client.addr == *except {
                continue;
            }
        }

        to_send.push(client.addr);
    }

    net.send_message_to_multiple(payload, to_send).ok();
}

fn process_payload(
    addr: SocketAddr,
    payload: Payloads,
    state: &mut ServerState,
    net: &mut SenderReceiver,
) {
    match &payload {
        // Unused
        Payloads::InvalidName { .. }
        | Payloads::RendezvousHandshake { .. }
        | Payloads::InvalidVersion { .. }
        | Payloads::PlayerJoined { .. }
        | Payloads::HostingReceived { .. }
        | Payloads::SetHost { .. }
        | Payloads::AttemptConnection { .. }
        | Payloads::AttemptHosterConnection { .. }
        | Payloads::RequestHosting { .. }
        | Payloads::PeerEstablished { .. }
        | Payloads::ConnectionDenied { .. }
        | Payloads::Heartbeat
        | Payloads::PlayerLeft { .. } => return,
        // Used
        Payloads::AircraftDefinition { bytes } => {
            state.aircraft_definition = Some(bytes.clone());
            return;
        }
        Payloads::Update { .. } => {}
        Payloads::InitHandshake { name, version } => {
            if let Ok(version) = Version::from_str(version) {
                let server_version =
                    Version::from_str(&dotenv::var("MINIMUM_VERSION").unwrap()).unwrap();

                if version < server_version {
                    net.send_message(
                        Payloads::ConnectionDenied {
                            reason: format!("Minimum version required is {}", server_version),
                        },
                        addr,
                    )
                    .ok();
                    return;
                }
            }

            if state.clients.contains_key(name) {
                net.send_message(Payloads::InvalidName {}, addr).ok();
                return;
            }

            // Send all current connected clients
            for (name, info) in state.clients.iter() {
                net.send_message(
                    Payloads::PlayerJoined {
                        name: name.clone(),
                        in_control: state.in_control == *name,
                        is_server: info.is_host,
                        is_observer: info.is_observer,
                    },
                    addr,
                )
                .ok();
            }

            // Add client
            state.clients.insert(name.clone(), Client::new(addr));

            // If the client is the first one to connect, give them control and have them "host"
            if state.in_control == SERVER_NAME {
                set_host(name.clone(), state, net);
            }

            send_to_all(
                Payloads::PlayerJoined {
                    name: name.clone(),
                    in_control: false,
                    is_server: false,
                    is_observer: false,
                },
                Some(&addr),
                state,
                net,
            );

            // Send definitions to new client
            if let Some(bytes) = state.aircraft_definition.as_ref() {
                net.send_message(
                    Payloads::AircraftDefinition {
                        bytes: bytes.clone(),
                    },
                    addr,
                )
                .ok();
            }

            info!("{} connected to hoster.", name);

            return;
        }
        Payloads::TransferControl { from: _, to } => {
            state.in_control = to.clone();
        }
        Payloads::SetObserver {
            from: _,
            to,
            is_observer,
        } => {
            if let Some(client) = state.clients.get_mut(to) {
                client.is_observer = *is_observer;
            }
        }
        Payloads::SetSelfObserver { name } => {
            if let Some(client) = state.clients.get_mut(name) {
                client.is_observer = true;
                send_to_all(
                    Payloads::SetObserver {
                        from: "SERVER".to_string(),
                        to: name.clone(),
                        is_observer: true,
                    },
                    None,
                    state,
                    net,
                );
            }
            return;
        }
        Payloads::Ready => {
            // Tell "host" to do a full sync
            if let Some(client) = state.clients.get(&state.in_control) {
                net.send_message(payload, client.addr).ok();
            }

            return;
        }
        Payloads::Handshake { session_id } => {
            info!("Hoster handshake received {}", session_id);
            net.send_message(payload, addr).ok();
            return;
        }
    }

    send_to_all(payload, Some(&addr), state, net);
}

fn set_host(name: String, state: &mut ServerState, net: &mut SenderReceiver) {
    let client = state.clients.get_mut(&name).expect("always there");
    client.is_observer = false;
    client.is_host = true;

    net.send_message(Payloads::SetHost, client.addr).ok();
    send_to_all(
        Payloads::TransferControl {
            from: state.in_control.clone(),
            to: name.clone(),
        },
        None,
        state,
        net,
    );

    state.in_control = name;
}

fn handle_heartbeats(servers: &mut HashMap<String, ServerState>, net: &mut SenderReceiver) {
    for (_, state) in servers.iter_mut() {
        if state.heartbeat_instant.elapsed().as_secs_f32() < 0.5 {
            return;
        }

        send_to_all(Payloads::Heartbeat, None, state, net);

        state.heartbeat_instant = Instant::now();
    }
}

fn cleanup(servers: &mut Servers) {
    let server_states = &mut servers.server_states;
    let active_servers = &mut servers.meta_state.active_servers;

    server_states.retain(|x, y| {
        if y.started_at.elapsed().as_secs() > INACTIVE_SESSION_TIMEOUT && y.clients.is_empty() {
            active_servers.remove(x);
            false
        } else {
            true
        }
    });

    servers
        .meta_state
        .clients_connected
        .retain(|_, session_id| server_states.get(session_id).is_some());
}

pub fn run_hoster(servers: Arc<Mutex<Servers>>, port: u16) {
    let socket = Socket::from_udp_socket(get_socket_duplex(port), get_socket_config(5))
        .expect("Failed to bind!");

    info!(
        "Hoster started on {}! Connect hostname {}",
        socket.local_addr().unwrap(),
        dotenv::var("SERVER_HOSTNAME").unwrap()
    );

    let mut net = SenderReceiver::from_socket(socket);

    let mut metrics_data = HashMap::new();

    let mut cleanup_timer = Instant::now();

    loop {
        net.poll();

        let mut servers = servers.lock().unwrap();

        loop {
            match net.get_next_message() {
                Ok(Message::Payload(addr, payload)) => {
                    // Handle initial connection verification
                    if let Payloads::Handshake { session_id } = &payload {
                        if let Some(previous_session_id) =
                            servers.meta_state.unknown_clients.get(&addr.ip()).cloned()
                        {
                            if session_id == &previous_session_id {
                                servers.meta_state.unknown_clients.remove(&addr.ip());
                                servers
                                    .meta_state
                                    .clients_connected
                                    .insert(addr, previous_session_id);
                            } else {
                                continue;
                            }
                        }
                    }

                    // Get server state for session
                    if let Some(session) = servers.meta_state.clients_connected.get(&addr) {
                        let session = session.clone();

                        if let Some(state) = servers.server_states.get_mut(&session) {
                            process_payload(addr, payload, state, &mut net);
                        }
                    }
                }
                Ok(Message::ConnectionClosed(addr)) => {
                    let client_removed = servers.meta_state.clients_connected.remove(&addr);

                    if let Some(session) = client_removed {
                        let mut should_close = false;

                        if let Some(state) = servers.server_states.get_mut(&session) {
                            let mut removed_name = String::new();
                            state.clients.retain(|name, client| {
                                if client.addr != addr {
                                    true
                                } else {
                                    removed_name = name.clone();
                                    false
                                }
                            });

                            // If was in control... need to transfer it to someone else or give it back to ourselves
                            if !state.clients.is_empty() {
                                if removed_name == state.in_control {
                                    let next = state.clients.iter().next().unwrap().0.clone();
                                    set_host(next, state, &mut net);
                                }
                            } else {
                                // Close server
                                should_close = true;
                                state.in_control = SERVER_NAME.to_string();
                            }

                            send_to_all(
                                Payloads::PlayerLeft { name: removed_name },
                                None,
                                state,
                                &mut net,
                            );
                        }

                        if should_close {
                            servers.remove_server(&session);
                        }
                    }

                    metrics_data.remove(&addr);
                }
                Ok(Message::Metrics(addr, metrics)) => {
                    metrics_data.insert(addr, metrics);
                }
                Err(Error::ReadTimeout(_)) => break,
                _ => {}
            }
        }

        if cleanup_timer.elapsed().as_secs() > CLEANUP_INTERVAL {
            cleanup(&mut servers);
            cleanup_timer = Instant::now();
        }

        handle_heartbeats(&mut servers.server_states, &mut net);

        drop(servers);

        sleep(Duration::from_millis(1));
    }
}
