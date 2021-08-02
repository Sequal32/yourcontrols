table! {
    lobbies (id) {
        id -> Int4,
        name -> Text,
        password -> Nullable<Text>,
        player_count -> Int4,
        refresh_key -> Text,
        private_address -> Text,
        public_address -> Text,
        created_at -> Timestamp,
        heartbeat_at -> Timestamp,
    }
}
