use actix_web::{web, App, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Arc;
use std::time::Instant;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{fmt, EnvFilter};

mod api;
mod config;
mod crypto;
mod errors;
mod jobs;
mod middleware;
mod model;
mod repository;
mod schema;
mod service;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct AppState {
    pub db_pool: DbPool,
    pub config: config::AppConfig,
    pub start_time: Instant,
}

fn run_migrations(pool: &DbPool) {
    let mut conn = pool
        .get()
        .expect("Failed to get DB connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
    tracing::info!("Database migrations applied successfully");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with JSON output and env-filter
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = config::AppConfig::load();

    let manager = ConnectionManager::<PgConnection>::new(&cfg.database.url);
    let pool = r2d2::Pool::builder()
        .max_size(cfg.database.max_connections)
        .build(manager)
        .expect("Failed to create database connection pool");

    run_migrations(&pool);

    // Spawn background job runner
    jobs::spawn_job_runner(pool.clone());

    let bind_addr = format!("{}:{}", cfg.server.bind_address, cfg.server.bind_port);
    tracing::info!("Starting server on {}", bind_addr);

    let state = Arc::new(AppState {
        db_pool: pool,
        config: cfg,
        start_time: Instant::now(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(state.clone()))
            .app_data(web::JsonConfig::default().limit(10 * 1024 * 1024))
            .wrap(TracingLogger::default())
            .configure(api::configure_routes)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
