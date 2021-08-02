use std::net::SocketAddr;

use database::establish_connection;
use dotenv::dotenv;
use util::{handle_rejection, with_auth};
use warp::Filter;

mod handlers;
mod models;
mod routes;
mod util;

#[tokio::main]
pub async fn main() {
    dotenv().ok();

    //  Helper filters

    let database_pool = establish_connection();
    let with_database = warp::any().map(move || database_pool.get().unwrap());

    let with_database_auth = with_database.clone().and(with_auth());

    // Routes

    let get_lobbies = routes::get_lobbies()
        .and(with_database.clone())
        .and_then(handlers::get_lobbies);

    let auth_lobby = routes::auth_lobby()
        .and(with_database.clone())
        .and_then(handlers::auth_lobby);

    let add_lobby = routes::add_lobby()
        .and(with_database_auth.clone())
        .and_then(handlers::add_lobby);

    let delete_lobby = routes::delete_lobby()
        .and(with_database_auth.clone())
        .and_then(handlers::delete_lobby);

    let filter = auth_lobby
        .or(add_lobby)
        .or(delete_lobby)
        .or(get_lobbies)
        // .or(not_found)
        .recover(handle_rejection);

    warp::serve(filter)
        .bind(
            dotenv::var("API_BIND_ADDRESS")
                .expect("should've been set")
                .parse::<SocketAddr>()
                .expect("should've been valid"),
        )
        .await;
}
