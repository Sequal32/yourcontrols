use anyhow::Result;
use diesel::{EqAll, Insertable, PgConnection, QueryDsl, RunQueryDsl};

use crate::models::{AddLobby, Lobby, LobbyJoinInfo, LobbyPublic};
use crate::schema::lobbies::dsl::*;

impl AddLobby {
    pub fn create(&self, conn: &PgConnection) -> Result<Lobby> {
        Ok(self.insert_into(lobbies).get_result(conn)?)
    }
}

impl Lobby {
    pub fn get(with_id: i32, conn: &PgConnection) -> Result<Lobby> {
        Ok(lobbies.filter(id.eq_all(with_id)).first(conn)?)
    }

    pub fn get_password(with_id: i32, conn: &PgConnection) -> Result<Option<String>> {
        Ok(lobbies
            .filter(id.eq_all(with_id))
            .select(password)
            .first(conn)?)
    }

    pub fn get_session_join_info(with_id: i32, conn: &PgConnection) -> Result<LobbyJoinInfo> {
        Ok(lobbies
            .filter(id.eq_all(with_id))
            .select((public_address, private_address))
            .first(conn)?)
    }

    pub fn delete(with_id: i32, conn: &PgConnection) -> Result<usize> {
        Ok(diesel::delete(lobbies.filter(id.eq_all(with_id))).execute(conn)?)
    }
}

impl LobbyPublic {
    pub fn get_list(conn: &PgConnection, limit: i64) -> Result<Vec<Self>> {
        Ok(lobbies
            .limit(limit)
            .select((id, name, player_count))
            .load(conn)?)
    }
}
