mod sessions;

use std::net::SocketAddr;

use anyhow::Result;
use sessions::Sessions;
use yourcontrols_net::{BaseSocket, MainPayloads};

pub struct RendezvousServer {
    socket: BaseSocket,
    sessions: Sessions,
}

impl RendezvousServer {
    pub fn new(port: u16) -> Result<Self> {
        Ok(Self {
            socket: BaseSocket::start(port)?,
            sessions: Sessions::new(),
        })
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
