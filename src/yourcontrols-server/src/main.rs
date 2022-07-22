use hoster::run_hoster;
use rendezvous::run_rendezvous;
use servers::Servers;
use simplelog::{LevelFilter, TermLogger, TerminalMode};
use std::sync::{Arc, Mutex};
use std::thread;

mod hoster;
mod rendezvous;
mod servers;
mod sessions;
mod util;

pub fn main() {
    dotenv::dotenv().ok();

    TermLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .ok();

    let servers = Arc::new(Mutex::new(Servers::new()));
    let servers_clone = servers.clone();

    thread::spawn(|| run_hoster(servers_clone, 5556));
    run_rendezvous(servers, 5555);
}
