mod hoster;

use dotenv::var;
use hoster::Hoster;
use yourcontrols_net::get_rendezvous_server;

fn main() {
    let mut hoster = Hoster::new(var("HOSTER_PORT").expect("PORT MISSING IN ENV").parse().unwrap(), get_rendezvous_server(false).unwrap());
    hoster.run();
}