pub mod users;
pub mod sessions;
pub mod resources;
pub mod lodgings;
pub mod inventory;
pub mod audit;
pub mod media;
pub mod import_jobs;
pub mod connector;
pub mod export;
pub mod config;

use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// Generic row type for raw SQL queries that return JSON.
#[derive(diesel::QueryableByName, Debug)]
pub struct JsonRow {
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub doc: serde_json::Value,
}
