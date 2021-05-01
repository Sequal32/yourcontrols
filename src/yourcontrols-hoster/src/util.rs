use std::net::SocketAddr;

use base64;
use rand::prelude::SliceRandom;

const LETTERS: &[u8; 26] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn get_random_id(length: u8) -> String {
    let mut rng = rand::thread_rng();

    let mut code = String::new();
    for _ in 0..length {
        code.push(LETTERS.choose(&mut rng).map(|&x| x as char).unwrap());
    }

    return code;
}

pub fn encode_ip(ip: &SocketAddr) -> String {
    base64::encode(ip.to_string())
}
