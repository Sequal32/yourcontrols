-- Your SQL goes here
CREATE TABLE IF NOT EXISTS lobbies (
    id serial PRIMARY KEY,
    name TEXT NOT NULL,
    password TEXT,
    player_count INTEGER NOT NULL DEFAULT 0,
    refresh_key TEXT NOT NULL DEFAULT substr(md5(random()::text), 0, 25),
    private_address TEXT NOT NULL,
    public_address TEXT NOT NULL,
    created_at timestamp without time zone NOT NULL default (now() at time zone 'utc'),
    heartbeat_at timestamp without time zone NOT NULL default (now() at time zone 'utc')
);