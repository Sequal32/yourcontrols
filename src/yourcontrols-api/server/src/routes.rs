use database::models::AddLobby;
use warp::{filters::BoxedFilter, Filter};

use crate::models::AuthLobbyPayload;

pub fn lobby_path() -> BoxedFilter<()> {
    warp::path("lobbies").boxed()
}

pub fn get_lobbies() -> BoxedFilter<()> {
    lobby_path().and(warp::path::end()).and(warp::get()).boxed()
}

pub fn auth_lobby() -> BoxedFilter<(i32, AuthLobbyPayload)> {
    lobby_path()
        .and(warp::path!(i32 / "auth"))
        .and(warp::post())
        .and(warp::body::json())
        .boxed()
}

pub fn add_lobby() -> BoxedFilter<(AddLobby,)> {
    lobby_path()
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .boxed()
}

pub fn delete_lobby() -> BoxedFilter<(i32,)> {
    lobby_path()
        .and(warp::path!(i32))
        .and(warp::path::end())
        .and(warp::delete())
        .boxed()
}
