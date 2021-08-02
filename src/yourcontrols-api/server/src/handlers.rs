use anyhow::Result;
use database::models::{AddLobby, Lobby, LobbyPublic};
use database::PgPooledConnection;
use warp::hyper::StatusCode;

use crate::models::{AuthLobbyPayload, InternalError};
use crate::util::{resolve_or_reject_payload, AnyhowError};

pub async fn get_lobbies(conn: PgPooledConnection) -> Result<impl warp::Reply, warp::Rejection> {
    resolve_or_reject_payload(LobbyPublic::get_list(&conn, 50))
}

pub async fn auth_lobby(
    lobby_id: i32,
    auth_payload: AuthLobbyPayload,
    conn: PgPooledConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    let password = match Lobby::get_password(lobby_id, &conn) {
        Ok(p) => p,
        Err(_) => return Err(warp::reject()),
    };

    if password != auth_payload.password {
        return Err(warp::reject::custom(InternalError {
            code: 403,
            reason: "Unauthorized".to_string(),
        }));
    }

    resolve_or_reject_payload(Lobby::get_session_join_info(lobby_id, &conn))
}

pub async fn add_lobby(
    add_payload: AddLobby,
    conn: PgPooledConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    resolve_or_reject_payload(add_payload.create(&conn))
}

pub async fn delete_lobby(
    lobby_id: i32,
    conn: PgPooledConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    let removed = Lobby::delete(lobby_id, &conn).map_err(|e| AnyhowError::into_reject(e))?;

    if removed > 0 {
        Ok(warp::reply::with_status("", StatusCode::NO_CONTENT))
    } else {
        Err(warp::reject())
    }
}
