pub mod api;
pub mod config;
pub mod crypto;
pub mod errors;
pub mod jobs;
pub mod middleware;
pub mod model;
pub mod repository;
pub mod schema;
pub mod service;

use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Arc;
use std::time::Instant;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct AppState {
    pub db_pool: DbPool,
    pub config: config::AppConfig,
    pub start_time: Instant,
}

pub fn run_migrations(pool: &DbPool) {
    let mut conn = pool
        .get()
        .expect("Failed to get DB connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
}

pub fn build_pool(database_url: &str, max_connections: u32) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(max_connections)
        .build(manager)
        .expect("Failed to create database connection pool")
}

/// Re-export the require_role macro so test crates can use it.
#[macro_export]
macro_rules! require_role {
    ($ctx:expr, $($role:ident),+ $(,)?) => {
        $ctx.require_any_role(&[$( $crate::model::UserRole::$role ),+])?;
    };
}
