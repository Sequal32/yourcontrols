use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{Value};
use std::net::{SocketAddr, IpAddr, TcpStream, Ipv4Addr};
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

    pub fn start(&self, ip: Ipv4Addr, port: u16) -> Result<(Sender<Value>, Receiver<ReceiveData>), std::io::Error>  {
        let (servertx, serverrx) = unbounded::<Value>();
        let (clienttx, clientrx) = unbounded::<ReceiveData>();

        let mut stream = TcpStream::connect_timeout(&SocketAddr::new(IpAddr::V4(ip), port), std::time::Duration::from_secs(5))?;
        let stream_clone = stream.try_clone().unwrap();

        let should_stop = self.should_stop.clone();
        let should_stop2 = self.should_stop.clone();

        thread::spawn(move || {
            thread::spawn(move || {
                loop {
                    if should_stop.load(SeqCst) {break}
                    // Send data to server
                    match serverrx.recv() {
                        Ok(data) => {
                            stream.write_all((data.to_string() + "\n").as_bytes()).expect("!");
                        }
                        Err(_) => break
                    }
                }
                
            });

            let mut reader = BufReader::new(&stream_clone);
            loop {
                let mut buf = String::new();
                if should_stop2.load(SeqCst) {break}
                // Send data to program
                match reader.read_line(&mut buf) {
                    Ok(_) => match serde_json::from_str(&buf.trim()) {
                        Ok(data) => clienttx.send(ReceiveData::Data(data)).expect("!"),
                        Err(_) => ()
                    },
                    Err(_) => break
                };
            }
        });
        

        return Ok((servertx, clientrx));
    }
}

impl TransferClient for Client {
    fn get_connected_count(&self) -> i32 {
        return 1;
    }

    fn stop(&self) {
        self.should_stop.store(true, SeqCst);
    }
}