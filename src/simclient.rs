use crossbeam_channel::{Sender, Receiver, unbounded};
use serde_json::{Value};
use std::net::{SocketAddr, IpAddr, TcpStream, Ipv4Addr};
use std::io::{Write, Read, BufReader, BufRead};
use std::thread;
use std::str::FromStr;

pub struct Client {}

impl Client {
    pub fn start(ip: Ipv4Addr, port: u16) -> Result<(Sender<Value>, Receiver<Value>), &'static str>  {
        let (servertx, serverrx) = unbounded::<Value>();
        let (clienttx, clientrx) = unbounded::<Value>();

        let mut stream = match TcpStream::connect(SocketAddr::new(IpAddr::V4(ip), port)) {
            Ok(stream) => stream,
            Err(_) => {return Err("Error opening stream!")}
        };

        thread::spawn(move || {
            thread::spawn(move || {
                loop {
                    // Send data to server
                    match serverrx.recv() {
                        Ok(data) => {
                            stream.write_all(data.to_string().as_bytes());
                        }
                        Err(_) => {}
                    }
                }
                
            });

            loop {
                let mut buf = String::new();
                let mut reader = BufReader::new(&stream);
                // Send data to program
                match reader.read_line(&mut buf) {
                    Ok(_) => match serde_json::from_str(&buf.trim()) {
                        Ok(data) => clienttx.send(data).expect("!"),
                        Err(_) => ()
                    },
                    Err(_) => ()
                };
            }
        });
        

        return Ok((servertx, clientrx));
    }
}