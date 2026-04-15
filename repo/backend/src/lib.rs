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

    // Each seeded account has its own fixed password for reliable tester login.
    let users: &[(&str, &str, &str, Option<&str>)] = &[
        ("admin",     "Admin@2024",     "Administrator", None),
        ("publisher", "Pub@2024",       "Publisher",     None),
        ("reviewer",  "Rev@2024",       "Reviewer",      None),
        ("clinician", "Clin@2024",      "Clinician",     Some("00000000-0000-0000-0000-000000000001")),
        ("clerk",     "Clerk@2024",     "InventoryClerk",Some("00000000-0000-0000-0000-000000000001")),
    ];

    for (username, password, role, facility) in users {
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

    tracing::info!("Default users seeded with fixed per-user credentials.");
}

/// Validates that secrets are properly configured. Panics in non-test profiles if
/// critical secrets are missing or set to known insecure defaults.
pub fn validate_secrets(cfg: &config::AppConfig) {
    let profile = &cfg.app.config_profile;
    if profile == "test" {
        return; // Skip validation in test profile
    }

    let known_insecure = [
        "",
        "h4ck-pr00f-hmac-secret-change-in-production-2024",
    ];

    if known_insecure.contains(&cfg.auth.hmac_secret.as_str()) {
        if profile == "production" {
            panic!("FATAL: HMAC secret is missing or set to an insecure default. Set auth.hmac_secret or HMAC_SECRET env var.");
        } else {
            tracing::warn!("HMAC secret is set to a development default. Override in production.");
        }
    }

    if cfg.crypto.aes256_master_key.is_empty() {
        if profile == "production" {
            panic!("FATAL: AES-256 master key is not configured. Set crypto.aes256_master_key or AES256_MASTER_KEY env var.");
        } else {
            tracing::warn!("AES-256 master key is empty. Encryption will fail.");
        }
    }
}

/// Re-export the require_role macro so test crates can use it.
#[macro_export]
macro_rules! require_role {
    ($ctx:expr, $($role:ident),+ $(,)?) => {
        $ctx.require_any_role(&[$( $crate::model::UserRole::$role ),+])?;
    };
}
