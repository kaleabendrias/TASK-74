use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use std::time::Instant;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

use tourism_backend::{api, build_pool, config, jobs, run_migrations, seed_defaults, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = config::AppConfig::load();
    let pool = build_pool(&cfg.database.url, cfg.database.max_connections);

    run_migrations(&pool);
    seed_defaults(&pool);
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
