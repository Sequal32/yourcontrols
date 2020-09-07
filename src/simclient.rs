use crossbeam_channel::{Sender, Receiver, bounded};
use serde_json::{Value};
use std::net::{SocketAddr, IpAddr, TcpStream};
use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
use std::io::{Write, BufReader, BufRead};
use std::thread;
use crate::simserver::{TransferClient, ReceiveData};

pub struct Client {
    should_stop: Arc<AtomicBool>
}

impl Client {
    pub fn new() -> Self {
        Self {
            should_stop: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn start(&self, ip: IpAddr, port: u16) -> Result<(Sender<Value>, Receiver<ReceiveData>), std::io::Error>  {
        let (servertx, serverrx) = bounded::<Value>(10);
        let (clienttx, clientrx) = bounded::<ReceiveData>(10);
        let tx2 = clienttx.clone();

        let mut stream = TcpStream::connect_timeout(&SocketAddr::new(ip, port), std::time::Duration::from_secs(5))?;
        let stream_clone = stream.try_clone().unwrap();

        let should_stop = self.should_stop.clone();
        let should_stop2 = self.should_stop.clone();

        thread::spawn(move || {
            thread::spawn(move || {
                loop {
                    // Send data to server
                    match serverrx.recv() {
                        Ok(data) => {
                            stream.write_all((data.to_string() + "\n").as_bytes()).expect("!");
                        }
                        Err(_) => {tx2.send(ReceiveData::TransferStopped("Connection lost".to_string())).ok();}
                    }
                    if should_stop.load(SeqCst) {break}
                }
            });

            let mut reader = BufReader::new(&stream_clone);
            loop {
                let mut buf = String::new();
                // Send data to program
                match reader.read_line(&mut buf) {
                    // Disconnected
                    Ok(0) => {
                        should_stop2.store(true, SeqCst);
                        clienttx.send(ReceiveData::TransferStopped("Connection lost".to_string())).ok();
                        break;
                    }
                    Ok(_) => match serde_json::from_str(&buf.trim()) {
                        Ok(data) => {clienttx.send(ReceiveData::Data(data)).ok();},
                        Err(_) => ()
                    },
                    // Reader error
                    Err(e) => {
                        should_stop2.store(true, SeqCst);
                        clienttx.send(ReceiveData::TransferStopped(e.to_string())).ok();
                    }
                };
                if should_stop2.load(SeqCst) {break}
            }
        });
        

        return Ok((servertx, clientrx));
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
}