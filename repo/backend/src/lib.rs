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

    let count: i64 = schema::users::table
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    if count > 0 {
        return;
    }

    // Check for INIT_TOKEN env var — if set, use it as the admin password
    let admin_pw = std::env::var("INIT_ADMIN_PASSWORD")
        .unwrap_or_else(|_| {
            let pw = crypto::csrf::generate_token()[..16].to_string();
            tracing::warn!("No INIT_ADMIN_PASSWORD set. Generated admin password: {}", pw);
            pw
        });

    tracing::info!("Seeding default facility and users...");

    diesel::sql_query(
        "INSERT INTO facilities (id, name, address) \
         VALUES ('00000000-0000-0000-0000-000000000001', 'Main Facility', '100 Tourism Blvd, Portal City, PC 00001') \
         ON CONFLICT DO NOTHING"
    ).execute(&mut conn).ok();

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

    // All seeded accounts use the same initial password (from INIT_ADMIN_PASSWORD or generated)
    let users = [
        ("admin",     "Administrator", None),
        ("publisher", "Publisher",      None),
        ("reviewer",  "Reviewer",       None),
        ("clinician", "Clinician",      Some("00000000-0000-0000-0000-000000000001")),
        ("clerk",     "InventoryClerk", Some("00000000-0000-0000-0000-000000000001")),
    ];

    for (username, role, facility) in &users {
        let hash = crypto::argon2id::hash(&admin_pw);
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

    tracing::info!("Default users seeded. All accounts use the initial password from INIT_ADMIN_PASSWORD or the generated value above.");
}

/// Re-export the require_role macro so test crates can use it.
#[macro_export]
macro_rules! require_role {
    ($ctx:expr, $($role:ident),+ $(,)?) => {
        $ctx.require_any_role(&[$( $crate::model::UserRole::$role ),+])?;
    };
}
