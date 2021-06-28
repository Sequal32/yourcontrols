#![cfg(test)]

use core::panic;

use crate::rendezvous::RendezvousServer;

use super::*;
use anyhow::Result;
use yourcontrols_net::{BaseSocket, ControlSurfaces};

struct SocketServerPair {
    server: SingleServer,
    socket: BaseSocket,
}

impl SocketServerPair {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: BaseSocket::start_with_bind_address("127.0.0.1:0")?,
            server: SingleServer::start_with_bind_address("127.0.0.1:0")?,
        })
    }

    /// Returns all Payloads
    pub fn poll_get_socket_messages(&mut self) -> Result<Vec<MainPayloads>> {
        self.socket.poll_no_receive();
        // Receive messages
        self.server.poll()?;
        // Send messages
        self.server.poll()?;
        Ok(self
            .socket
            .poll::<MainPayloads>()
            .into_iter()
            .filter_map(|x| match x {
                Message::Payload(m, _) => Some(m),
                _ => None,
            })
            .collect())
    }

    pub fn send_socket_message(&mut self, addr: SocketAddr, payload: MainPayloads) -> Result<()> {
        Ok(self.socket.send_to(addr, &payload)?)
    }

    pub fn server_address(&self) -> SocketAddr {
        self.server.get_address()
    }
}

fn test_name(net: &mut SocketServerPair, with_rendezvous: bool) -> Result<()> {
    let mut id = 0;

    net.send_socket_message(
        net.server_address(),
        MainPayloads::Name {
            name: "TEST".to_string(),
        },
    )?;

    // Expect MakeHost payload for first client connecting
    let messages = net.poll_get_socket_messages()?;

    // IN SEQUENCE
    let mut in_sequence_messages = messages.into_iter();

    // Get assigned client id
    let next_msg = in_sequence_messages.next().unwrap();
    match next_msg {
        MainPayloads::Welcome { client_id } => {
            id = client_id;
        }
        _ => panic!("Expected welcome payload, got {:?}", next_msg),
    }

    // Get assigned host
    let next_msg = in_sequence_messages.next().unwrap();
    match next_msg {
        MainPayloads::MakeHost { client_id } => {
            if id != client_id {
                panic!("Wrong client made host")
            }
        }
        _ => panic!("Expected host payload, got {:?}", next_msg),
    }

    // Check session id is sent back and is the same
    if with_rendezvous {
        let next_msg = in_sequence_messages.next().unwrap();
        match next_msg {
            MainPayloads::SessionDetails { session_id } => {
                if net.server.session_id().unwrap() != &session_id {
                    panic!("Wrong session id!")
                }
            }
            _ => panic!("Expected SessionDetails payload, got {:?}", next_msg),
        }
    }

    // All control surfaces should be delegated to this client
    let next_msg = in_sequence_messages.next().unwrap();
    match next_msg {
        MainPayloads::ControlDelegations { delegations } => {
            assert!(
                delegations.len() == ControlSurfaces::all().len(),
                "All control surfaces should've been set!"
            );

            if !delegations.values().all(|x| *x == id) {
                panic!("All control delegations should be given to us")
            }
        }
        _ => panic!("Expected ControlDelegations payload, got {:?}", next_msg),
    };

    // No other clients connected so no other payloads should be sent
    assert!(in_sequence_messages.next().is_none());

    Ok(())
}

fn test_hello(net: &mut SocketServerPair, with_rendezvous: bool) -> Result<()> {
    let session_id = if with_rendezvous {
        net.server.session_id().unwrap().clone()
    } else {
        String::new()
    };

    let payload = MainPayloads::Hello {
        session_id,
        version: String::new(),
    };

    net.send_socket_message(net.server_address(), payload)?;

    // Expect MakeHost payload for first client connecting
    let messages = net.poll_get_socket_messages()?;

    // Verify valid version
    for message in messages.iter() {
        match message {
            MainPayloads::InvalidVersion { .. } => {
                panic!("Version should've been valid")
            }
            MainPayloads::InvalidSession { .. } => {
                panic!("Session should've been valid")
            }
            _ => {}
        }
    }

    Ok(())
}

fn test_request_session(net: &mut SocketServerPair) -> Result<()> {
    let addr: SocketAddr = "127.0.0.1:25070".parse().unwrap();

    let mut rendezvous = RendezvousServer::start_with_bind_address(addr)?;

    net.server.set_rendezvous_server(addr);
    net.server.request_session()?;

    // Send session message
    net.poll_get_socket_messages()?;
    // Receive messages
    rendezvous.step()?;
    // Send messages
    rendezvous.step()?;
    // Receive session id
    net.poll_get_socket_messages()?;

    assert!(net.server.session_id().is_some());

    Ok(())
}

#[test]
fn test_sequence() -> Result<()> {
    let mut net = SocketServerPair::new()?;

    test_hello(&mut net, false)?;
    test_name(&mut net, false)?;

    Ok(())
}

#[test]
fn test_sequence_with_rendezvous() -> Result<()> {
    let mut net = SocketServerPair::new()?;

    test_request_session(&mut net)?;
    test_hello(&mut net, true)?;
    test_name(&mut net, true)?;

    Ok(())
}
