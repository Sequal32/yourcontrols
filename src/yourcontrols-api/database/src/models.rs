pub use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Insertable, Debug)]
#[table_name = "lobbies"]
#[serde(rename_all = "camelCase")]
pub struct AddLobby {
    pub name: String,
    pub password: String,
    pub private_address: String,
    pub public_address: String,
}

#[derive(Deserialize, Serialize, Queryable, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Lobby {
    pub id: i32,
    pub name: String,
    pub password: Option<String>,
    pub player_count: i32,
    pub refresh_key: String,
    pub private_address: String,
    pub public_address: String,
    pub created_at: NaiveDateTime,
    pub heartbeat_at: NaiveDateTime,
}

#[derive(Deserialize, Serialize, Queryable, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LobbyJoinInfo {
    pub private_address: String,
    pub public_address: String,
}

#[derive(Deserialize, Serialize, Queryable, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LobbyPublic {
    pub id: i32,
    pub name: String,
    pub player_count: i32,
}
