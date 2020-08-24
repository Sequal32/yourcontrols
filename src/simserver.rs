use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{Value};
use std::io::{Write, Read};
use std::net::{TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;


pub struct Server {
}

impl Server {
    pub fn new() -> Self  {
        return Self {
            
        }
    }

    pub fn start(&mut self, port: u16) -> Result<(Sender<Value>, Receiver<Value>), std::io::Error> {
        let (servertx, serverrx) = unbounded::<Value>();
        let (clienttx, clientrx) = unbounded::<Value>();

        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(listener) => listener,
            Err(n) => {return Err(n)}
        };

        thread::spawn(move || {
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
                    let addr = stream.peer_addr().unwrap();

                    println!("NEW CONNECTION {}", addr.ip().to_string());

                    // Create clones of streams that each thread can use safely
                    let mut stream_clone = stream.try_clone().unwrap();

                    // Will determine thread disconnection
                    let did_disconnect = Arc::new(AtomicBool::new(false));
                    let disconnect_clone = did_disconnect.clone();

                    thread::spawn(move || {
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
                                    // Receive data
                                    tx.send(serde_json::from_str(&String::from_utf8(buf[..n].to_vec()).unwrap()).unwrap()).expect("Error transmitting data!");
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