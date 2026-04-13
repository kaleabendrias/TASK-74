use actix_web::{web, App, test as actix_test};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use std::sync::Arc;
use std::time::Instant;

use tourism_backend::{api, build_pool, run_migrations, AppState, DbPool};
use tourism_backend::config::*;
use tourism_backend::crypto::argon2id;
use tourism_backend::schema::*;

pub fn test_config() -> AppConfig {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tourism:tourism_secret_2024@localhost:5433/tourism_portal_test".into());
    AppConfig {
        server: ServerConfig { bind_address: "127.0.0.1".into(), bind_port: 0 },
        database: DatabaseConfig { url: db_url, max_connections: 5, min_connections: 1, connect_timeout_secs: 5 },
        tls: TlsConfig { cert_path: "/dev/null".into(), key_path: "/dev/null".into() },
        auth: AuthConfig {
            hmac_secret: "test-hmac-secret".into(),
            request_signing_key: "req-sign-key-tourism-portal-2024".into(),
            session_ttl_secs: 28800,
            csrf_token_ttl_secs: 3600,
            argon2: Argon2Config { memory_kib: 4096, iterations: 1, parallelism: 1, output_len: 32 },
        },
        crypto: CryptoConfig { aes256_master_key: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [0x42u8; 32]) },
        totp: TotpConfig { issuer: "Test".into(), digits: 6, period_secs: 30 },
        uploads: UploadConfig { max_size_bytes: 52428800, allowed_mimes: vec!["image/jpeg".into(), "image/png".into()], storage_path: "/tmp/test_uploads".into() },
        features: FeatureFlags { mfa_enabled: true, csv_import: true, export_watermark: true, lodging_deposit_cap: true, canary_release: false },
        maintenance: MaintenanceConfig { window_cron: "0 3 * * 0".into() },
        prometheus: PrometheusConfig { scrape_path: "/metrics".into() },
        canary: CanaryConfig { profile: "stable".into() },
        app: AppMetaConfig { config_profile: "test".into(), service_name: "test-backend".into(), version: "0.1.0-test".into() },
    }
}

pub fn setup_pool() -> DbPool {
    let cfg = test_config();
    let pool = build_pool(&cfg.database.url, cfg.database.max_connections);
    run_migrations(&pool);
    pool
}

pub fn create_test_app(pool: DbPool) -> (Arc<AppState>, actix_web::dev::ServiceResponse) {
    // just return the state for now
    unreachable!()
}

pub fn get_state(pool: DbPool) -> Arc<AppState> {
    Arc::new(AppState {
        db_pool: pool,
        config: test_config(),
        start_time: Instant::now(),
    })
}

pub fn seed_users(pool: &DbPool) -> SeedData {
    let mut conn = pool.get().expect("get conn");

    // Clean tables in dependency order
    diesel::sql_query("DELETE FROM audit_log").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM export_approvals").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM import_jobs").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM api_connector_logs").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM idempotency_keys").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM review_decisions").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM media_files").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM inventory_transactions").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM inventory_lots").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM bins").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM warehouses").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM lodging_rent_changes").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM lodging_periods").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM lodgings").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM resource_versions").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM resources").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM csrf_tokens").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM sessions").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM users").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM facilities").execute(&mut conn).ok();
    diesel::sql_query("DELETE FROM config_parameters").execute(&mut conn).ok();

    // Create facility
    let facility_id: uuid::Uuid = diesel::sql_query(
        "INSERT INTO facilities (id, name, address) VALUES (gen_random_uuid(), 'Test Facility', '123 Test St') RETURNING id"
    ).get_result::<FacilityId>(&mut conn).unwrap().id;

    // Create warehouse and bin
    let warehouse_id: uuid::Uuid = diesel::sql_query(
        &format!("INSERT INTO warehouses (id, facility_id, name) VALUES (gen_random_uuid(), '{}', 'Warehouse A') RETURNING id", facility_id)
    ).get_result::<WarehouseId>(&mut conn).unwrap().id;

    let bin_id: uuid::Uuid = diesel::sql_query(
        &format!("INSERT INTO bins (id, warehouse_id, label) VALUES (gen_random_uuid(), '{}', 'Bin-01') RETURNING id", warehouse_id)
    ).get_result::<BinId>(&mut conn).unwrap().id;

    let pw = argon2id::hash("testpassword");

    // Admin user
    let admin_id = insert_user(&mut conn, "admin", &pw, "Administrator", None);
    let publisher_id = insert_user(&mut conn, "publisher", &pw, "Publisher", None);
    let reviewer_id = insert_user(&mut conn, "reviewer", &pw, "Reviewer", None);
    let clinician_id = insert_user(&mut conn, "clinician", &pw, "Clinician", Some(facility_id));
    let clerk_id = insert_user(&mut conn, "clerk", &pw, "InventoryClerk", Some(facility_id));

    SeedData {
        facility_id, warehouse_id, bin_id,
        admin_id, publisher_id, reviewer_id, clinician_id, clerk_id,
    }
}

fn insert_user(conn: &mut PgConnection, username: &str, pw_hash: &str, role: &str, facility_id: Option<uuid::Uuid>) -> uuid::Uuid {
    let fid = facility_id.map(|f| format!("'{}'", f)).unwrap_or("NULL".into());
    let q = format!(
        "INSERT INTO users (id, username, password_hash, role, facility_id, mfa_enabled) \
         VALUES (gen_random_uuid(), '{}', '{}', '{}', {}, false) RETURNING id",
        username, pw_hash, role, fid
    );
    diesel::sql_query(&q).get_result::<UserId>(conn).unwrap().id
}

#[derive(QueryableByName)]
pub struct FacilityId {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: uuid::Uuid,
}

#[derive(QueryableByName)]
pub struct WarehouseId {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: uuid::Uuid,
}

#[derive(QueryableByName)]
pub struct BinId {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: uuid::Uuid,
}

#[derive(QueryableByName)]
pub struct UserId {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: uuid::Uuid,
}

pub struct SeedData {
    pub facility_id: uuid::Uuid,
    pub warehouse_id: uuid::Uuid,
    pub bin_id: uuid::Uuid,
    pub admin_id: uuid::Uuid,
    pub publisher_id: uuid::Uuid,
    pub reviewer_id: uuid::Uuid,
    pub clinician_id: uuid::Uuid,
    pub clerk_id: uuid::Uuid,
}

/// Returns base URL of the running test server.
pub fn base_url() -> String {
    std::env::var("TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into())
}

/// Login helper — extracts session token from Set-Cookie header and returns it
/// along with the CSRF token. Uses Bearer auth for subsequent requests since
/// the session cookie has Secure flag which won't work over plain HTTP in tests.
pub async fn login_as(client: &reqwest::Client, username: &str) -> (String, String) {
    let resp = client.post(&format!("{}/api/auth/login", base_url()))
        .json(&serde_json::json!({"username": username, "password": "testpassword"}))
        .send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Login failed for {}", username);

    // Extract session token from Set-Cookie header
    let session = resp.headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            if s.starts_with("session=") {
                Some(s.split(';').next()?.trim_start_matches("session=").to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let body: serde_json::Value = resp.json().await.unwrap();
    let csrf = body["csrf_token"].as_str().unwrap_or("").to_string();
    (session, csrf)
}

/// Creates a client that injects the Bearer token into every request.
/// Accepts self-signed certs for TLS test environments.
pub fn bearer_client(session_token: &str) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", session_token)).unwrap(),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

/// Creates a plain client that accepts self-signed certs.
pub fn authed_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}
