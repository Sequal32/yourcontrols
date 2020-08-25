use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{Value};
use std::net::{SocketAddr, IpAddr, TcpStream, Ipv4Addr};
use std::io::{Write, BufReader, BufRead};
use std::thread;
use crate::simserver::{TransferClient, ReceiveData};

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self, ip: Ipv4Addr, port: u16) -> Result<(Sender<Value>, Receiver<ReceiveData>), &'static str>  {
        let (servertx, serverrx) = unbounded::<Value>();
        let (clienttx, clientrx) = unbounded::<ReceiveData>();

        let mut stream = match TcpStream::connect(SocketAddr::new(IpAddr::V4(ip), port)) {
            Ok(stream) => stream,
            Err(_) => {return Err("Error opening stream!")}
        };
        let stream_clone = stream.try_clone().unwrap();

        thread::spawn(move || {
            thread::spawn(move || {
                loop {
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
}