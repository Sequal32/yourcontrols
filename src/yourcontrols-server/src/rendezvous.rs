use crate::servers::Servers;
use crate::sessions::Sessions;
use crate::util::Counters;
use laminar::Socket;
use log::info;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use yourcontrols_net::{get_socket_config, Message, Payloads, SenderReceiver};
use yourcontrols_types::Error;

const MAX_REQUESTS_PER_HOUR: u32 = 300;

fn process_message(
    addr: SocketAddr,
    message: Payloads,
    net: &mut SenderReceiver,
    sessions: &mut Sessions,
    counters: &mut Counters,
    servers: &mut Arc<Mutex<Servers>>,
) {
    match message {
        Payloads::RendezvousHandshake {
            session_id,
            local_endpoint,
        } => {
            let state = &mut servers.lock().unwrap().meta_state;

            if let Some(server_connection_info) = sessions.get_session_connection_info(&session_id)
            {
                // SELF HOSTED SESSION
                info!(
                    "{} wants to join {}",
                    counters.get_id_for_addr(&addr.ip()),
                    session_id
                );
                // Send data to client
                net.send_message(
                    Payloads::AttemptConnection {
                        peers: server_connection_info.hoster_endpoints.clone(),
                    },
                    addr,
                )
                .ok();

                // Send data to hoster
                net.send_message(
                    Payloads::AttemptConnection {
                        peers: vec![local_endpoint, Some(addr)]
                            .into_iter()
                            .flatten()
                            .collect(),
                    },
                    server_connection_info.hoster_addr,
                )
                .ok();

                sessions.add_client_to_session(session_id, addr);
            } else if let Some(server_info) = state.active_servers.get(&session_id) {
                // HOSTED SESSION
                state.unknown_clients.insert(addr.ip(), session_id);

                net.send_message(
                    Payloads::AttemptHosterConnection {
                        peer: server_info.address,
                    },
                    addr,
                )
                .ok();
            } else {
                // Invalid session
                net.send_message(
                    Payloads::ConnectionDenied {
                        reason: "Session not found.".to_string(),
                    },
                    addr,
                )
                .ok();
            }
        }

        Payloads::PeerEstablished { peer } => {
            info!(
                "{} established connection with {}",
                counters.get_id_for_addr(&addr.ip()),
                counters.get_id_for_addr(&peer.ip())
            );
        }

        Payloads::RequestHosting {
            self_hosted,
            local_endpoint,
        } => {
            let session_id;

            if self_hosted {
                session_id = sessions.map_session_id_to_socket_info(
                    addr,
                    vec![local_endpoint, Some(addr)]
                        .into_iter()
                        .flatten()
                        .collect(),
                );
                info!(
                    "Self hosted session created with hoster {} as {}",
                    counters.get_id_for_addr(&addr.ip()),
                    session_id
                );
            } else {
                let mut servers = servers.lock().unwrap();
                // Limit
                if servers.meta_state.clients_connected.len()
                    >= dotenv::var("MAX_CLIENT_CONNECTIONS")
                        .unwrap()
                        .parse()
                        .unwrap()
                {
                    net.send_message(
                        Payloads::ConnectionDenied {
                            reason: "Server at capacity.".to_string(),
                        },
                        addr,
                    )
                    .ok();
                    return;
                }
                // Reserve
                let hoster_addr: SocketAddr = dotenv::var("HOSTER_IP")
                    .expect("should be set")
                    .parse()
                    .unwrap();
                session_id = servers.reserve_server(hoster_addr, addr);

                info!(
                    "Hosting session for hoster {} as {}",
                    counters.get_id_for_addr(&addr.ip()),
                    session_id
                );
                // Tell client to handshake with the hoster
                net.send_message(
                    Payloads::AttemptConnection {
                        peers: vec![hoster_addr],
                    },
                    addr,
                )
                .ok();
            }

            net.send_message(Payloads::HostingReceived { session_id }, addr)
                .ok();
        }

        _ => {}
    }
}

pub fn run_rendezvous(servers: Arc<Mutex<Servers>>, port: u16) {
    let socket = Socket::bind_with_config(
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port),
        get_socket_config(3),
    )
    .expect("Failed to bind!");
    let mut net = SenderReceiver::from_socket(socket);
    let mut sessions = Sessions::new();

    let mut counters = Counters::new();
    let mut info_timer = Instant::now();
    let mut servers = servers;

    info!("Server started on port {}!", port);

    loop {
        net.poll();

        loop {
            match net.get_next_message() {
                Ok(Message::Payload(addr, message)) => {
                    // More than 300 requests within the last hour... probably malicious intent
                    if counters.get_request_count_for(&addr.ip()) > MAX_REQUESTS_PER_HOUR {
                        continue;
                    }
                    process_message(
                        addr,
                        message,
                        &mut net,
                        &mut sessions,
                        &mut counters,
                        &mut servers,
                    );
                    counters.increment_request_counter(addr.ip());
                }
                Ok(Message::ConnectionClosed(addr)) => {
                    let ip = addr.ip();

                    if sessions.socket_is_hosting(&addr) {
                        info!("{} lost connection, and was hosting. Closing session {:?}. Was connected for {} seconds.", counters.get_id_for_addr(&ip), sessions.close_session_by_addr(&addr), counters.get_last_request_seconds(&ip));
                    } else {
                        info!("{} lost connection, was in session {:?}. Was connected for {} seconds.", counters.get_id_for_addr(&ip), sessions.remove_client_from_session(&addr), counters.get_last_request_seconds(&ip));
                    }
                }
                Err(Error::NetEncodeError(_)) => {
                    // warn!("{} sent invalid data! {:?}", addr, string_data);
                    // counters.increment_request_counter(addr.ip());
                    // TODO: blacklist client
                }
                Err(Error::ReadTimeout(_)) => {
                    break;
                }
                _ => {}
            };
        }

        if info_timer.elapsed().as_secs() >= 600 {
            let servers = servers.lock().unwrap();
            info!(
                "Connections: {}, Sessions: {}, Hosted Sessions: {}, Clients In Hosted: {}",
                sessions.get_user_count(),
                sessions.get_session_count(),
                servers.meta_state.active_servers.len(),
                servers.meta_state.clients_connected.len()
            );

            info_timer = Instant::now();
            counters.cleanup();
        }

        sleep(Duration::from_millis(10));
    }
}
