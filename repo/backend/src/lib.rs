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

use diesel::prelude::*;
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

/// Runs all pending Diesel database migrations against the given connection pool.
pub fn run_migrations(pool: &DbPool) {
    let mut conn = pool
        .get()
        .expect("Failed to get DB connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
}

/// Creates an r2d2 connection pool for PostgreSQL with the specified max connections.
pub fn build_pool(database_url: &str, max_connections: u32) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(max_connections)
        .build(manager)
        .expect("Failed to create database connection pool")
}

/// Seeds default users and a facility if the users table is empty.
/// Called once at startup to ensure the portal is usable out of the box.
pub fn seed_defaults(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get DB connection for seeding");

    // Check if users exist
    let count: i64 = schema::users::table
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    if count > 0 {
        return;
    }

    tracing::info!("Seeding default facility and users...");

    // Create a default facility
    diesel::sql_query(
        "INSERT INTO facilities (id, name, address) \
         VALUES ('00000000-0000-0000-0000-000000000001', 'Main Facility', '100 Tourism Blvd, Portal City, PC 00001') \
         ON CONFLICT DO NOTHING"
    ).execute(&mut conn).ok();

    // Create a warehouse and bin for inventory
    diesel::sql_query(
        "INSERT INTO warehouses (id, facility_id, name) \
         VALUES ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 'Central Warehouse') \
         ON CONFLICT DO NOTHING"
    ).execute(&mut conn).ok();

    diesel::sql_query(
        "INSERT INTO bins (id, warehouse_id, label) \
         VALUES ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000002', 'A-01') \
         ON CONFLICT DO NOTHING"
    ).execute(&mut conn).ok();

    // Seed users with Argon2id-hashed passwords
    let users = [
        ("admin",     "admin123",     "Administrator", None),
        ("publisher", "publisher123", "Publisher",      None),
        ("reviewer",  "reviewer123",  "Reviewer",       None),
        ("clinician", "clinician123", "Clinician",      Some("00000000-0000-0000-0000-000000000001")),
        ("clerk",     "clerk123",     "InventoryClerk", Some("00000000-0000-0000-0000-000000000001")),
    ];

    for (username, password, role, facility) in &users {
        let hash = crypto::argon2id::hash(password);
        let fid = facility
            .map(|f| format!("'{}'", f))
            .unwrap_or_else(|| "NULL".to_string());
        let q = format!(
            "INSERT INTO users (id, username, password_hash, role, facility_id, mfa_enabled) \
             VALUES (gen_random_uuid(), '{}', '{}', '{}', {}, false) \
             ON CONFLICT (username) DO NOTHING",
            username,
            hash.replace('\'', "''"),
            role,
            fid
        );
        diesel::sql_query(&q).execute(&mut conn).ok();
    }

    tracing::info!("Default users seeded: admin, publisher, reviewer, clinician, clerk");
}

/// Re-export the require_role macro so test crates can use it.
#[macro_export]
macro_rules! require_role {
    ($ctx:expr, $($role:ident),+ $(,)?) => {
        $ctx.require_any_role(&[$( $crate::model::UserRole::$role ),+])?;
    };
}
