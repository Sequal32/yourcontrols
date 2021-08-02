#[macro_use]
extern crate diesel;

use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use diesel::PgConnection;
use std::env;

pub mod handlers;
pub mod models;
mod schema;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::new(database_url);
    Pool::builder().build(manager)
}

pub fn establish_connection() -> PgPool {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    init_pool(&database_url).expect("Could not connect to the database!")
}
