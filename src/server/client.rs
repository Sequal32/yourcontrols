use crossbeam_channel::{Receiver, Sender, unbounded};
use serde_json::{Value, json};
use std::{io::Read, net::{SocketAddr, IpAddr, TcpStream}, sync::Mutex};
use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
use std::io::Write;
use std::thread;

use super::{PartialReader, TransferStoppedReason, process_message, util::{ReceiveData, TransferClient}};

struct TransferStruct {
    stream: TcpStream,
    
    reader: PartialReader,
    // Internally receive data to send to clients
    client_rx: Receiver<Value>,
    // Send data to app to receive client data
    server_tx: Sender<ReceiveData>,
    
}

pub struct Client {
    should_stop: Arc<AtomicBool>,
    transfer: Option<Arc<Mutex<TransferStruct>>>,
    // Recieve data from clients
    server_rx: Receiver<ReceiveData>,
    // Send data to clients
    client_tx: Sender<Value>,
    // Internally receive data to send to clients
    client_rx: Receiver<Value>,
    // Send data to app to receive client data
    server_tx: Sender<ReceiveData>,
    // IP
    server_name: String
}

impl Client {
    pub fn new() -> Self {
        let (client_tx, client_rx) = unbounded();
        let (server_tx, server_rx) = unbounded();

        Self {
            should_stop: Arc::new(AtomicBool::new(false)),
            transfer: None,
            client_rx, client_tx, server_rx, server_tx,
            server_name: String::new()
        }
    }

    pub fn start(&mut self, ip: IpAddr, port: u16) -> Result<(), std::io::Error>  {
        let stream = TcpStream::connect_timeout(&SocketAddr::new(ip, port), std::time::Duration::from_secs(5))?;
        stream.set_nonblocking(true).unwrap();

        self.server_name = stream.local_addr().unwrap().to_string();

        // to be used in run()
        self.transfer = Some(Arc::new(Mutex::new(
            TransferStruct {
                stream: stream,
                reader: PartialReader::new(),
                client_rx: self.client_rx.clone(),
                server_tx: self.server_tx.clone(),
            }
        )));

        return Ok(());
    }

    pub fn run(&self) {
        let transfer = self.transfer.as_ref().unwrap().clone();
        let should_stop = self.should_stop.clone();

        self.send_name();

        thread::spawn(move || {
            loop {
                let transfer = &mut transfer.lock().unwrap();
    
                // Read data from server
                let mut buf = [0; 1024];
                match transfer.stream.read(&mut buf) {
                    // No data read, connection dropped
                    Ok(0) => {
                        transfer.server_tx.send(ReceiveData::TransferStopped(TransferStoppedReason::ConnectionLost)).ok();
                        should_stop.store(true, SeqCst);
                        break;
                    }
                    Ok(n) => {
                        if let Some(data) = transfer.reader.try_read_string(&buf[0..n]) {
                            // Deserialize json
                            if let Ok(data) = process_message(&data, None) {
                                // Server identified itself
                                if let ReceiveData::Name(name) = data {
                                    transfer.server_tx.send(ReceiveData::NewConnection(name)).ok();
                                } else {
                                    // Don't need to resend name to app
                                    transfer.server_tx.send(data).ok();
                                }   
                            }
                        }
                    }
                    Err(_) => {}
                };
    
                // Send data from app to clients
                if let Ok(data) = transfer.client_rx.try_recv() {
                    // Broadcast data to all clients
                    match transfer.stream.write_all((data.to_string() + "\n").as_bytes()) {
                        Ok(_) => {}
                        Err(e) => {
                            // Connection dropped
                            should_stop.store(true, SeqCst);
                            transfer.server_tx.send(ReceiveData::TransferStopped(TransferStoppedReason::ConnectionLost)).ok();
                            break
                        }
                    };
                }

                if should_stop.load(SeqCst) {break}
            }
        });
    }
}

impl TransferClient for Client {
    fn get_connected_count(&self) -> u16 {
        return 1;
    }

    fn stop(&self) {
        self.should_stop.store(true, SeqCst);
        self.server_tx.send(ReceiveData::TransferStopped(TransferStoppedReason::Requested)).ok();
    }

    fn stopped(&self) -> bool {
        self.should_stop.load(SeqCst)
    }

    fn is_server(&self) -> bool {
        false
    }

    fn get_transmitter(&self) -> &Sender<Value> {
        return &self.client_tx
    }

    fn get_receiver(&self) -> &Receiver<ReceiveData> {
        return &self.server_rx
    }

    fn get_server_name(&self) -> &str {
        return &self.server_name
    }
}