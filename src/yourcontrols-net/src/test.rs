#![cfg(test)]
use super::*;
use base::*;
use handshake::*;
use std::net::SocketAddr;

const SERVER_SESSION_ID: &str = "HOSTED";
const SELF_SESSION_ID: &str = "SELF";

fn get_socket() -> BaseSocket {
    BaseSocket::start_with_bind_address("127.0.0.1:0").unwrap()
}

fn poll_in_progress(handshake: Box<dyn Handshake>) -> Box<dyn Handshake> {
    match handshake.handshake() {
        Err(HandshakeFail::InProgress(h)) => h,
        _ => panic!("in progress"),
    }
}

struct RendezvousTester {
    socket: BaseSocket,
    target_address: SocketAddr,
}

impl RendezvousTester {
    fn new(socket: BaseSocket) -> Self {
        Self {
            target_address: socket.get_address(),
            socket,
        }
    }

    fn new_with_target(socket: BaseSocket, target_address: SocketAddr) -> Self {
        Self {
            socket,
            target_address,
        }
    }

    fn encode_address(&self) -> String {
        base64::encode(self.target_address.to_string())
    }

    pub fn poll(&mut self) {
        for message in self.socket.poll::<HandshakePayloads>() {
            match message {
                Message::Payload(HandshakePayloads::Hello { .. }, addr) => {
                    self.socket
                        .send_to(
                            addr,
                            &HandshakePayloads::AttemptConnection {
                                public_ip: self.encode_address(),
                                local_ip: self.encode_address(),
                            },
                        )
                        .unwrap();
                }
                Message::Payload(HandshakePayloads::RequestSession { self_hosted }, addr) => {
                    self.socket
                        .send_to(
                            addr,
                            &HandshakePayloads::SessionDetails {
                                session_id: if self_hosted {
                                    SELF_SESSION_ID.to_string()
                                } else {
                                    SERVER_SESSION_ID.to_string()
                                },
                            },
                        )
                        .unwrap();
                }
                _ => {}
            }
        }
    }
}

struct DirectTester {
    socket: BaseSocket,
    invalid_version: bool,
}

impl DirectTester {
    pub fn new(socket: BaseSocket, invalid_version: bool) -> Self {
        Self {
            socket,
            invalid_version,
        }
    }

    pub fn poll(&mut self) {
        for message in self.socket.poll::<HandshakePayloads>() {
            match message {
                Message::Payload(
                    HandshakePayloads::Hello {
                        session_id,
                        version,
                    },
                    addr,
                ) => {
                    self.socket
                        .send_to(
                            addr,
                            &if self.invalid_version {
                                HandshakePayloads::InvalidVersion
                            } else {
                                HandshakePayloads::Hello {
                                    session_id,
                                    version,
                                }
                            },
                        )
                        .unwrap();
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test_direct_handshake() {
    let client_socket = get_socket();
    let mut server = DirectTester::new(get_socket(), false);

    let handshake = Box::new(DirectHandshake::new(
        client_socket,
        HandshakeConfig::default(),
        server.socket.get_address(),
        None,
    ));

    // Start handshaking to server
    let handshake = poll_in_progress(handshake);

    // Receive client message and send hello back
    server.poll();
    // Actually send the message
    server.poll();

    assert!(handshake.handshake().is_ok(), "should've connected");
}

#[test]
fn test_request_hosting() {
    let client_socket = get_socket();
    let mut server = RendezvousTester::new(get_socket());

    let handshake = Box::new(SessionHostHandshake::new(
        client_socket,
        HandshakeConfig {
            rendezvous_address: server.socket.get_address(),
            self_hosted: false,
            ..Default::default()
        },
    ));

    // Start handshaking to server
    let handshake = poll_in_progress(handshake);

    // Receive message
    server.poll();
    // Send message
    server.poll();

    // Receive session ID
    let handshake = handshake.handshake().expect("completed");
    assert_eq!(handshake.get_session_id(), SERVER_SESSION_ID);
}

#[test]
fn test_self_hosting() {
    let client_socket = get_socket();
    let mut server = RendezvousTester::new(get_socket());

    let handshake = Box::new(SessionHostHandshake::new(
        client_socket,
        HandshakeConfig {
            rendezvous_address: server.socket.get_address(),
            self_hosted: true,
            ..Default::default()
        },
    ));

    // Start handshaking to server
    let handshake = poll_in_progress(handshake);

    // Receive message
    server.poll();
    // Send message
    server.poll();

    // Receive session ID
    let handshake = handshake.handshake().expect("completed");
    assert_eq!(handshake.get_session_id(), SELF_SESSION_ID);
}

#[test]
fn test_punchthrough_local() {
    let client_socket = get_socket();

    let mut other_client = DirectTester::new(get_socket(), false);
    let mut server =
        RendezvousTester::new_with_target(get_socket(), other_client.socket.get_address());

    let handshake = Box::new(PunchthroughHandshake::new(
        client_socket,
        HandshakeConfig {
            rendezvous_address: server.socket.get_address(),
            self_hosted: true,
            ..Default::default()
        },
    ));

    let handshake = poll_in_progress(handshake);
    println!("{}", server.socket.get_address());

    // Receive session code
    server.poll();
    // Send IPs to attempt to connect to
    server.poll();

    // Receive connection details
    let handshake = poll_in_progress(handshake);
    // Send message to connect to other client
    let handshake = poll_in_progress(handshake);

    // Other client receives
    other_client.poll();
    // Other client responds
    other_client.poll();

    // Should receive message and complete handshake
    handshake.handshake().expect("complete");
}
