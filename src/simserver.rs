use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{Value};
use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering::SeqCst};
use std::thread;

pub trait TransferClient {
    fn get_connected_count(&self) -> u16;
    fn stop(&self);
}

pub struct Server {
    pub number_connections: Arc<AtomicU16>,
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
            number_connections: Arc::new(AtomicU16::new(0)),
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
        listener.set_nonblocking(true).ok();

        thread::spawn(move || {
            loop {
                if let Ok((stream, addr)) = listener.accept() {
                    // Create sender/receivers that each thread can use safely

                    let tx = servertx.clone();
                    let tx2 = tx.clone();
                    let rx = clientrx.clone();
                    // Address to send udp packets with
                    let addr = Arc::new(addr.to_string());
                    let addr_clone = addr.clone();
                    let addr_clone2 = addr.clone();
                    tx.send(ReceiveData::NewConnection(addr.to_string())).expect("!");

                    // Create clones of streams that each thread can use safely
                    let mut stream_clone = stream.try_clone().unwrap();

                    // Will determine thread disconnection
                    let did_disconnect = Arc::new(AtomicBool::new(false));
                    let disconnect_clone = did_disconnect.clone();

                    let mut reader = BufReader::new(stream.try_clone().unwrap());

                    let number_connections = number_connections.clone();
                    number_connections.fetch_add(1, SeqCst);

                    thread::spawn(move || {
                        loop {
                            let mut buf = String::new();

                            match reader.read_line(&mut buf) {
                                // socket closed
                                Ok(n) => {
                                    if n == 0 {
                                        did_disconnect.store(true, SeqCst);
                                        number_connections.fetch_min(1, SeqCst);
                                        tx.send(ReceiveData::ConnectionLost(addr_clone.to_string())).ok();
                                        break
                                    }
                                    // Receive data
                                    tx.send(ReceiveData::Data(serde_json::from_str(&buf).unwrap())).expect("Error transmitting data!");
                                },
                                Err(e) => {
                                    did_disconnect.store(true, SeqCst);
                                    number_connections.fetch_min(1, SeqCst);
                                    tx.send(ReceiveData::ConnectionLost(addr_clone.to_string())).ok();
                                    break
                                }
                            }
                        }
                    });

                    thread::spawn(move || {
                        loop {
                            // Send to all clients
                            let data = rx.recv();
                            if disconnect_clone.load(SeqCst) {break}
    
                            match data {
                                Ok(value) => {
                                    let payload_string = value.to_string() + "\n";
                                    let payload = payload_string.as_bytes();
                                    match stream_clone.write(payload) {
                                        Ok(_) => (),
                                        Err(_) => {
                                            tx2.send(ReceiveData::ConnectionLost(addr_clone2.to_string())).expect("Error transmitting data (client)!");
                                            break
                                        }
                                    }
                                },
                                Err(_) => break
                            }
                        }
                    });
                }

                if should_stop.load(SeqCst) {break}
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        

        return Ok((clienttx, serverrx));
    }
}

impl TransferClient for Server {
    fn get_connected_count(&self) -> u16 {
        return self.number_connections.load(SeqCst);
    }

    fn stop(&self) {
        self.should_stop.store(true, SeqCst);
    }
}