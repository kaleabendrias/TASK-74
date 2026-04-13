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

use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;
