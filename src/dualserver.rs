use std::net::{TcpListener, UdpSocket};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{json, Value};
use std::io::{Write, Read};

pub struct ControllingState {
}

pub struct DualServer {
}

pub enum OpCodes {
    Update(Value),
    UpdatePeriodical(Value),
    Init(Value),
    RequestControl,
    RelieveControl,
}

fn handle_data(data: &Value) -> Result<OpCodes, &'static str> {
    match data["op"].as_str() {
        Some("update") => Ok(OpCodes::Update(data["payload"].clone())),
        Some("periodical") => Ok(OpCodes::UpdatePeriodical(data["payload"].clone())),
        Some("init") => Ok(OpCodes::Init(data["payload"].clone())),
        Some("request") => Ok(OpCodes::RequestControl),
        Some("relieve") => Ok(OpCodes::RelieveControl),
        _ => Err("Undefined opcode.")
    }
}

impl DualServer {
    pub fn new() -> Self  {
        return Self {
            
        }
    }

    pub fn start(&mut self, port: u32) -> (Sender<Value>, Receiver<Value>) {
        let (servertx, serverrx) = unbounded::<Value>();
        let (clienttx, clientrx) = unbounded::<Value>();

        
        std::thread::spawn(move || {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
            let udp = UdpSocket::bind(format!("127.0.0.1:{}", port + 1)).unwrap();

            loop {
                for stream in listener.incoming() {
                    let mut stream = match stream {
                        Ok(stream) => stream,
                        Err(_) => continue
                    };

                    // Create sender/receivers that each thread can use safely

                    let tx = servertx.clone();
                    let rx = clientrx.clone();
                    // Address to send udp packets with
                    let addr = stream.local_addr().unwrap();

                    // Create clones of streams that each thread can use safely
                    let mut stream_clone = stream.try_clone().unwrap();
                    let udpclone = udp.try_clone().unwrap();

                    // Will determine thread disconnection
                    let did_disconnect = Arc::new(AtomicBool::new(false));
                    let disconnect_clone = did_disconnect.clone();

                    std::thread::spawn(move || {
                        loop {
                            let mut buf = [0; 1024];
                            let tcpdata = stream.read(&mut buf);

                            match tcpdata {
                                // socket closed
                                Ok(n) if n == 0 => {
                                    did_disconnect.store(true, Ordering::SeqCst);
                                    break
                                },
                                Ok(n) => {
                                    tx.send(serde_json::from_str(&String::from_utf8(buf[..n].to_vec()).unwrap()).unwrap()).expect("Error transmitting data!");
                                },
                                Err(_) => ()
                            }
                        }
                    });
                    
                    std::thread::spawn(move || {
                        loop {
                            let data = rx.recv();

                            if disconnect_clone.load(Ordering::SeqCst) {break}

                            match data {
                                Ok(value) => {
                                    let payload_string = value["payload"].to_string();
                                    let payload = payload_string.as_bytes();
                                    match value["type"].as_str().unwrap() {
                                        "udp" => udpclone.send_to(payload, addr),
                                        "tcp" => stream_clone.write(payload),
                                        _ => panic!("Undefined value type!")
                                    }.expect("Error receiving data!");
                                },
                                Err(_) => break
                            }
                        }
                    });
                }
            }
        });
        

        return (clienttx, serverrx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_recv() {
        let mut server = DualServer::new();
        let (tx, rx) = server.start(32005);
        loop {
            // tx.send(Value::from("[\"hello\"]"));
            println!("{:?}", rx.recv());
            // std::thread::sleep_ms(1000);
        }
    }
}