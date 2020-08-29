use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{Value};
use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::SeqCst};
use std::thread;

pub trait TransferClient {
    fn get_connected_count(&self) -> i32;
    fn stop(&self);
}

pub struct Server {
    pub number_connections: Arc<AtomicI32>,
    should_stop: Arc<AtomicBool>
}

pub enum ReceiveData {
    Data(Value),
    NewConnection(String),
    ConnectionLost(String)
}

impl Server {
    pub fn new() -> Self  {
        return Self {
            number_connections: Arc::new(AtomicI32::new(0)),
            should_stop: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn start(&mut self, port: u16) -> Result<(Sender<Value>, Receiver<ReceiveData>), std::io::Error> {
        let (servertx, serverrx) = unbounded::<ReceiveData>();
        let (clienttx, clientrx) = unbounded::<Value>();

        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(listener) => listener,
            Err(n) => {return Err(n)}
        };

        let number_connections = self.number_connections.clone();
        let should_stop = self.should_stop.clone();
        let should_stop2 = should_stop.clone();

        thread::spawn(move || {
            loop {
                for stream in listener.incoming() {
                    let stream = match stream {
                        Ok(stream) => stream,
                        Err(_) => continue
                    };

                    // Create sender/receivers that each thread can use safely

                    let tx = servertx.clone();
                    let tx2 = tx.clone();
                    let rx = clientrx.clone();
                    // Address to send udp packets with
                    let addr = stream.peer_addr().unwrap().ip().to_string();
                    tx.send(ReceiveData::NewConnection(addr.to_string())).expect("!");

                    // Create clones of streams that each thread can use safely
                    let mut stream_clone = stream.try_clone().unwrap();

                    // Will determine thread disconnection
                    let did_disconnect = Arc::new(AtomicBool::new(false));
                    let should_stop = should_stop.clone();
                    let should_stop2 = should_stop.clone();
                    let disconnect_clone = did_disconnect.clone();

                    let mut reader = BufReader::new(stream);

                    let number_connections = number_connections.clone();
                    number_connections.fetch_add(1, SeqCst);

                    thread::spawn(move || {
                        loop {
                            let mut buf = String::new();

                            if should_stop.load(SeqCst) {break}

                            match reader.read_line(&mut buf) {
                                // socket closed
                                Ok(n) => {
                                    if n == 0 {
                                        did_disconnect.store(true, SeqCst);
                                        number_connections.fetch_min(1, SeqCst);
                                        break
                                    }
                                    // Receive data
                                    tx.send(ReceiveData::Data(serde_json::from_str(&buf).unwrap())).expect("Error transmitting data!");
                                },
                                Err(_) => {
                                    did_disconnect.store(true, SeqCst);
                                    number_connections.fetch_min(1, SeqCst);
                                    break
                                }
                            }
                        }
                    });

                    thread::spawn(move || {
                        loop {
                            // Send to all clients
                            let data = rx.recv();
                            if disconnect_clone.load(SeqCst) || should_stop2.load(SeqCst) {break}
    
                            match data {
                                Ok(value) => {
                                    let payload_string = value.to_string() + "\n";
                                    let payload = payload_string.as_bytes();
                                    match stream_clone.write(payload) {
                                        Ok(_) => (),
                                        Err(_) => {
                                            tx2.send(ReceiveData::ConnectionLost(addr.to_string())).expect("Error transmitting data (client)!");
                                            break
                                        }
                                    }
                                },
                                Err(_) => break
                            }
                        }
                    });
                    
                    
                }
            }
        });
        

        return Ok((clienttx, serverrx));
    }
}

impl TransferClient for Server {
    fn get_connected_count(&self) -> i32 {
        return self.number_connections.load(SeqCst);
    }

    fn stop(&self) {
        self.should_stop.store(true, SeqCst);
    }
}