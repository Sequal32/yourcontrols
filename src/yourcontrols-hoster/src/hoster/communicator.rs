use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub struct Communicator {
    target_addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl Communicator {
    pub fn new(target_addr: SocketAddr) -> Self {
        Self {
            target_addr,
            stream: None,
        }
    }

    fn is_established(&self) -> bool {
        return self.stream.is_some();
    }

    fn establish_connection(&mut self) {
        if self.is_established() {
            return;
        }

        let stream = match TcpStream::connect_timeout(&self.target_addr, Duration::from_secs(1)) {
            Ok(s) => s,
            Err(_) => return,
        };

        stream.set_nonblocking(true).ok();

        self.stream = Some(stream);
    }

    fn read_message(&mut self) -> Result<HosterPayloads, ()> {
        let mut stream = match &mut self.stream {
            Some(s) => BufReader::new(s),
            None => return Err(()),
        };

        let mut buf = Vec::new();
        stream.read_until(0x0A, &mut buf).ok();

        buf.pop(); // Pop delimeter byte

        match rmp_serde::decode::from_read_ref(&buf) {
            Ok(p) => Ok(p),
            Err(_) => Err(()),
        }
    }

    pub fn send_message(&mut self, payload: HosterPayloads) {
        let stream = match &mut self.stream {
            Some(s) => s,
            None => return,
        };

        let mut bytes = match rmp_serde::encode::to_vec(&payload) {
            Ok(b) => b,
            Err(_) => return,
        };

        bytes.push(0x0A);

        stream.write_all(&bytes).ok();
    }

    pub fn poll(&mut self) -> Result<HosterPayloads, ()> {
        self.establish_connection();
        self.read_message()
    }
}

#[derive(Serialize, Deserialize)]
pub enum HosterPayloads {
    SessionOpen {
        session_id: String,
    },
    SessionClosed {
        session_id: String,
    },
    ClientAcknowledged {
        addr: SocketAddr,
    },
    HostingRequested {
        session_id: String,
    },
    ClientConnecting {
        session_id: String,
        addr: SocketAddr,
    },
}
