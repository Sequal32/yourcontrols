// mod hoster;
mod rendezvous;
mod util;

use std::{thread::sleep, time::Duration};

use dotenv::var;
use rendezvous::RendezvousServer;
use yourcontrols_types::Result;
// use hoster::Hoster;
// use yourcontrols_net::get_rendezvous_server;

fn main() -> Result<()> {
    dotenv::dotenv().expect("env file missing");

    let mut server = RendezvousServer::new(
        var("RENDEZVOUS_PORT")
            .expect("port missing in env")
            .parse()
            .expect("not a number"),
    )?;

    // Main loop
    loop {
        server.step()?;
        sleep(Duration::from_millis(10));
    }

    // let mut hoster = Hoster::new(
    //     var("HOSTER_PORT")
    //         .expect("PORT MISSING IN ENV")
    //         .parse()
    //         .unwrap(),
    //     get_rendezvous_server(false).unwrap(),
    // );
    // hoster.run();

    Ok(())
}
