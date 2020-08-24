use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{Value};
use std::io::{Write, Read, BufReader, BufRead};
use std::net::{TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;


pub struct Server {
}

pub enum ReceiveData {
    Data(Value),
    NewConnection(String),
}

impl Server {
    pub fn new() -> Self  {
        return Self {
            
        }
    }

    pub fn start(&mut self, port: u16) -> Result<(Sender<Value>, Receiver<ReceiveData>), std::io::Error> {
        let (servertx, serverrx) = unbounded::<ReceiveData>();
        let (clienttx, clientrx) = unbounded::<Value>();

        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(listener) => listener,
            Err(n) => {return Err(n)}
        };

        thread::spawn(move || {
            loop {
                for stream in listener.incoming() {
                    let stream = match stream {
                        Ok(stream) => stream,
                        Err(_) => continue
                    };

                    // Create sender/receivers that each thread can use safely

                    let tx = servertx.clone();
                    let rx = clientrx.clone();
                    // Address to send udp packets with
                    let addr = stream.peer_addr().unwrap().ip().to_string();
                    tx.send(ReceiveData::NewConnection(addr)).expect("!");

                    // Create clones of streams that each thread can use safely
                    let mut stream_clone = stream.try_clone().unwrap();

                    // Will determine thread disconnection
                    let did_disconnect = Arc::new(AtomicBool::new(false));
                    let disconnect_clone = did_disconnect.clone();

                    let mut reader = BufReader::new(stream);

                    thread::spawn(move || {
                        loop {
                            let mut buf = String::new();

                            match reader.read_line(&mut buf) {
                                // socket closed
                                Ok(n) => {
                                    if n == 0 {
                                        did_disconnect.store(true, Ordering::SeqCst);
                                        break
                                    }
                                    // Receive data
                                    tx.send(ReceiveData::Data(serde_json::from_str(&buf).unwrap())).expect("Error transmitting data!");
                                },
                                Err(_) => ()
                            }
                        }
                    });
                    
                    loop {
                        // Send to all clients
                        let data = rx.recv();

                        if disconnect_clone.load(Ordering::SeqCst) {break}

                        match data {
                            Ok(value) => {
                                let payload_string = value.to_string() + "\n";
                                let payload = payload_string.as_bytes();
                                stream_clone.write(payload).expect("!");
                            },
                            Err(_) => break
                        }
                    }
                }
            }
        });
        

        return Ok((clienttx, serverrx));
    }
}