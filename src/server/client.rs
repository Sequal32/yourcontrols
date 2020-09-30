use crossbeam_channel::{Receiver, Sender, unbounded};
use serde_json::{Value, json};
use std::{io::Read, net::{SocketAddr, IpAddr, TcpStream}, sync::Mutex};
use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
use std::io::Write;
use std::thread;
use crate::{definitions::AllNeedSync};

use super::{server::PartialReader, util::{ControlTransferType, ReceiveData, TransferClient}};

struct TransferStruct {
    stream: TcpStream,
    
    reader: PartialReader,
    // Send data to clients
    client_tx: Sender<Value>,
    // Internally receive data to send to clients
    client_rx: Receiver<Value>,

    // Send data to app to receive client data
    server_tx: Sender<ReceiveData>,
    // Recieve data from clients
    server_rx: Receiver<ReceiveData>,
}

pub struct Client {
    should_stop: Arc<AtomicBool>,
    transfer: Option<Arc<Mutex<TransferStruct>>>
}

impl Client {
    pub fn new() -> Self {
        Self {
            should_stop: Arc::new(AtomicBool::new(false)),
            transfer: None
        }
    }

    pub fn start(&mut self, ip: IpAddr, port: u16) -> Result<(Sender<Value>, Receiver<ReceiveData>), std::io::Error>  {
        let stream = TcpStream::connect_timeout(&SocketAddr::new(ip, port), std::time::Duration::from_secs(5))?;
        stream.set_nonblocking(true).unwrap();

        let (client_tx, client_rx) = unbounded();
        let (server_tx, server_rx) = unbounded();

        // to be used in run()
        self.transfer = Some(Arc::new(Mutex::new(
            TransferStruct {
                stream: stream,
                reader: PartialReader::new(),
                client_tx: client_tx.clone(),
                client_rx: client_rx.clone(),
                server_tx: server_tx.clone(),
                server_rx: server_rx.clone(),
            }
        )));

        return Ok((client_tx, server_rx));
    }

    pub fn run(&self) {
        let transfer = self.transfer.as_ref().unwrap().clone();
        let should_stop = self.should_stop.clone();

        thread::spawn(move || {
            loop {
                let transfer = &mut transfer.lock().unwrap();
    
                // Read data from server
                let mut buf = [0; 1024];
                transfer.stream.read(&mut buf);

                if let Some(data) = transfer.reader.try_read_string(&buf) {
                    // Deserialize json
                    if let Ok(data) = serde_json::from_str(data.as_str()) {
                        transfer.server_tx.send(ReceiveData::Data(data));
                    }
                }
    
                // Send data from app to clients
                if let Ok(data) = transfer.client_rx.try_recv() {
                    // Broadcast data to all clients
                    match transfer.stream.write_all((data.to_string() + "\n").as_bytes()) {
                        Ok(_) => {}
                        Err(_) => {
                            // Connection dropped
                            should_stop.store(true, SeqCst);
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
    }

    fn stopped(&self) -> bool {
        self.should_stop.load(SeqCst)
    }

    fn is_server(&self) -> bool {
        false
    }

    fn change_control(&self, control_type: ControlTransferType) {
        match control_type {
            ControlTransferType::Take => {
                self.send_value(json!({
                    "type":"take_control"
                }));
            }
            ControlTransferType::Relieve => {
                self.send_value(json!({
                    "type":"relieve_control"
                }));
            }
            ControlTransferType::Cancel => {
                self.send_value(json!({
                    "type":"cancel_relieve"
                }));
            }
        }
    }

    fn send_value(&self, message: Value) {
        self.transfer.as_ref().unwrap().lock().unwrap().client_tx.send(
            message
        ).ok();
    }

    fn update(&self, data: AllNeedSync) {
        self.send_value(serde_json::to_value(data).unwrap());
    }
}