mod sessions;

use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::Result;
use laminar::Socket;
use sessions::Sessions;
use yourcontrols_net::{BaseSocket, MainPayloads};

pub struct RendezvousServer {
    socket: BaseSocket,
    sessions: Sessions,
}

impl RendezvousServer {
    pub fn start() -> Result<Self> {
        Self::start_with_port(0)
    }

    pub fn start_with_port(port: u16) -> Result<Self> {
        Ok(Self::start_with_socket(BaseSocket::start_with_port(port)?))
    }

    pub fn start_with_bind_address(address: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self::start_with_socket(
            BaseSocket::start_with_bind_address(address)?,
        ))
    }

    fn start_with_socket(socket: BaseSocket) -> Self {
        Self {
            socket,
            sessions: Sessions::new(),
        }
    }

    pub fn get_address(&self) -> SocketAddr {
        self.socket.get_address()
    }

    pub fn process_payload(&mut self, addr: SocketAddr, message: MainPayloads) -> Result<()> {
        match message {
            // Used
            MainPayloads::Hello {
                session_id,
                version,
            } => {
                if let Some(session_info) = self.sessions.get_info_for_session(&session_id) {
                    self.socket.send_to(
                        addr,
                        &MainPayloads::AttemptConnection {
                            public_ip: session_info.public.clone(),
                            local_ip: session_info.private.clone(),
                        },
                    )?;
                }
            }
            MainPayloads::RequestSession { self_hosted } => {
                if self_hosted {
                    let session_id = self.sessions.map_session_id_to_socket_info(addr);

                    self.socket
                        .send_to(addr, &MainPayloads::SessionDetails { session_id })?;
                } else {
                    // Send connection to hoster
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<()> {
        for message in self.socket.poll() {
            match message {
                yourcontrols_net::Message::NewConnection(addr) => {}
                yourcontrols_net::Message::LostConnection(addr) => {}
                yourcontrols_net::Message::Payload(payload, addr) => {
                    self.process_payload(addr, payload)?
                }
            }
        }

        Ok(())
    }
}
