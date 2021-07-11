#![cfg(test)]

use core::panic;

use crate::rendezvous::RendezvousServer;

use super::*;
use anyhow::Result;
use yourcontrols_net::{BaseSocket, ControlSurfaces};

const VERSION: &str = "1.0.0";

struct SocketServerTriplet {
    server: SingleServer,
    socket_1: BaseSocket,
    socket_2: BaseSocket,
}

impl SocketServerTriplet {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket_1: BaseSocket::start_with_bind_address("127.0.0.1:0")?,
            socket_2: BaseSocket::start_with_bind_address("127.0.0.1:0")?,
            server: SingleServer::start_with_bind_address("127.0.0.1:0")?,
        })
    }

    fn get_payloads_from_socket(socket: &mut BaseSocket) -> Result<Vec<MainPayloads>> {
        Ok(socket
            .poll::<MainPayloads>()
            .into_iter()
            .filter_map(|x| match x {
                Message::Payload(m, _) => Some(m),
                _ => None,
            })
            .collect())
    }

    fn double_poll_server(&mut self) -> Result<()> {
        // Receive messages
        self.server.poll()?;
        // Send messages
        self.server.poll()?;

        Ok(())
    }

    pub fn poll_get_socket_1_messages(&mut self) -> Result<Vec<MainPayloads>> {
        self.socket_1.poll_no_receive();
        self.double_poll_server()?;
        Self::get_payloads_from_socket(&mut self.socket_1)
    }

    pub fn poll_get_socket_2_messages(&mut self) -> Result<Vec<MainPayloads>> {
        self.socket_2.poll_no_receive();
        self.double_poll_server()?;
        Self::get_payloads_from_socket(&mut self.socket_2)
    }

    pub fn poll_get_socket_messages(&mut self) -> Result<(Vec<MainPayloads>, Vec<MainPayloads>)> {
        self.socket_1.poll_no_receive();
        self.socket_2.poll_no_receive();
        self.double_poll_server()?;
        Ok((
            Self::get_payloads_from_socket(&mut self.socket_1)?,
            Self::get_payloads_from_socket(&mut self.socket_2)?,
        ))
    }

    pub fn send_socket_1_message(&mut self, addr: SocketAddr, payload: MainPayloads) -> Result<()> {
        Ok(self.socket_1.send_to(addr, &payload)?)
    }

    pub fn send_socket_2_message(&mut self, addr: SocketAddr, payload: MainPayloads) -> Result<()> {
        Ok(self.socket_2.send_to(addr, &payload)?)
    }

    pub fn server_address(&self) -> SocketAddr {
        self.server.get_address()
    }
}

struct SingleServerTester {
    net: SocketServerTriplet,
    rendezvous: RendezvousServer,
}

impl SingleServerTester {
    pub fn new() -> Result<Self> {
        let rendezvous = RendezvousServer::start()?;

        let mut rendezvous_address = rendezvous.get_address();
        rendezvous_address.set_ip("127.0.0.1".parse().unwrap());

        let mut net = SocketServerTriplet::new()?;
        net.server.set_rendezvous_server(rendezvous_address);
        net.server.set_version(VERSION.to_string());

        Ok(Self { rendezvous, net })
    }

    fn test_name(&mut self, with_rendezvous: bool) -> Result<()> {
        let mut id = 0;

        self.net.send_socket_1_message(
            self.net.server_address(),
            MainPayloads::Name {
                name: "TEST".to_string(),
            },
        )?;

        // Expect MakeHost payload for first client connecting
        let messages = self.net.poll_get_socket_1_messages()?;

        // IN SEQUENCE
        let mut in_sequence_messages = messages.into_iter();

        // Get assigned client id
        let next_msg = in_sequence_messages.next().unwrap();
        match next_msg {
            MainPayloads::Welcome { client_id, name } => {
                assert_eq!(&name, "TEST");
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
                    if self.net.server.session_id().unwrap() != &session_id {
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

    fn test_hello(&mut self, with_rendezvous: bool) -> Result<()> {
        let session_id = if with_rendezvous {
            self.net.server.session_id().unwrap().clone()
        } else {
            String::new()
        };

        let payload = MainPayloads::Hello {
            session_id,
            version: VERSION.to_string(),
        };

        self.net
            .send_socket_1_message(self.net.server_address(), payload)?;

        // Expect MakeHost payload for first client connecting
        let messages = self.net.poll_get_socket_1_messages()?;

        // Verify valid version
        let next_msg = messages.into_iter().next().unwrap();
        match next_msg {
            MainPayloads::InvalidVersion { .. } => {
                panic!("Version should've been valid")
            }
            MainPayloads::InvalidSession { .. } => {
                panic!("Session should've been valid")
            }
            MainPayloads::Hello { .. } => {}
            _ => panic!("Expected Hello payload, got {:?}", next_msg),
        };

        Ok(())
    }

    fn test_request_session(&mut self) -> Result<()> {
        self.net.server.request_session()?;

        // Send session message
        self.net.double_poll_server()?;
        // Receive messages
        self.rendezvous.step()?;
        // Send messages
        self.rendezvous.step()?;
        // Receive session id
        self.net.double_poll_server()?;

        assert!(self.net.server.session_id().is_some());

        Ok(())
    }
}

#[test]
fn test_sequence() -> Result<()> {
    let mut net = SingleServerTester::new()?;

    net.test_hello(false)?;
    net.test_name(false)?;

    Ok(())
}

#[test]
fn test_sequence_with_rendezvous() -> Result<()> {
    let mut net = SingleServerTester::new()?;

    net.test_request_session()?;
    net.test_hello(true)?;
    net.test_name(true)?;

    Ok(())
}
