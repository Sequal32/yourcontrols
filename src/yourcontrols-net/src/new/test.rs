#![cfg(test)]
use super::*;
use base::*;
use handshake::*;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn setup_sockets_and_addresses() -> ((BaseSocket, SocketAddr), (BaseSocket, SocketAddr)) {
    let server_socket = BaseSocket::start(0).unwrap();
    let client_socket = BaseSocket::start(0).unwrap();

    let mut server_address = server_socket.get_address();
    let mut client_address = client_socket.get_address();

    server_address.set_ip(IpAddr::V4(Ipv4Addr::LOCALHOST));
    client_address.set_ip(IpAddr::V4(Ipv4Addr::LOCALHOST));

    (
        (server_socket, server_address),
        (client_socket, client_address),
    )
}

#[test]
fn test_direct_handshake() {
    let ((mut server_socket, server_address), (client_socket, client_address)) =
        setup_sockets_and_addresses();

    let handshake = Box::new(DirectHandshake::new(
        client_socket,
        HandshakeConfig::default(),
        server_address,
        None,
    ));

    // Start handshaking to server
    let handshake = match handshake.handshake() {
        Err(HandshakeFail::InProgress(h)) => h,
        _ => panic!("in progress"),
    };

    // Receive client message and send hello back
    for message in server_socket.poll::<HandshakePayloads>() {
        match message {
            Message::Payload(payload) => {
                server_socket.send_to(client_address, &payload).ok();
            }
            _ => panic!("should've received client Hello."),
        }
    }
    // Actually send the message
    server_socket.poll::<()>();

    assert!(handshake.handshake().is_ok(), "should've connected");
}
