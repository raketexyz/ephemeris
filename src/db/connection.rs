use crate::ApiError;
use diesel::{r2d2::ConnectionManager, PgConnection};
use lazy_static::lazy_static;
use std::env;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

lazy_static! {
    static ref POOL: Pool = Pool::new(ConnectionManager::<PgConnection>::new(
        env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    ))
    .unwrap_or_else(|e| panic!("Couldn't create db pool: {}", e));
}

pub fn init() {
    info!("Creating db pool");
    lazy_static::initialize(&POOL);
    connection().unwrap_or_else(|e| panic!("Connection failed: {}", e));
}

pub fn connection() -> Result<DbConnection, ApiError> {
    POOL.get()
        .map_err(|e| ApiError::new(500, format!("Couldn't get db connection: {}", e)))
}
