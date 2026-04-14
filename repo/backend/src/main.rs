use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use std::time::Instant;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

use tourism_backend::{api, build_pool, config, jobs, run_migrations, seed_defaults, AppState};

fn load_rustls_config(cfg: &config::TlsConfig) -> Option<rustls::ServerConfig> {
    let cert_path = &cfg.cert_path;
    let key_path = &cfg.key_path;

    if cert_path == "/dev/null" || key_path == "/dev/null" {
        return None;
    }

    let cert_file = match std::fs::File::open(cert_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(error = %e, path = %cert_path, "TLS cert not found, starting without TLS");
            return None;
        }
    };
    let key_file = match std::fs::File::open(key_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(error = %e, path = %key_path, "TLS key not found, starting without TLS");
            return None;
        }
    };

    let certs: Vec<_> = rustls_pemfile::certs(&mut std::io::BufReader::new(cert_file))
        .filter_map(|r| r.ok())
        .collect();
    let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(key_file))
        .ok()
        .flatten();

    if certs.is_empty() {
        tracing::warn!("No certificates found in {}", cert_path);
        return None;
    }
    let key = match key {
        Some(k) => k,
        None => {
            tracing::warn!("No private key found in {}", key_path);
            return None;
        }
    };

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .ok()?;

    Some(tls_config)
}

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
    jobs::spawn_scheduled_publisher(pool.clone());

    let bind_addr = format!("{}:{}", cfg.server.bind_address, cfg.server.bind_port);
    let tls_config = load_rustls_config(&cfg.tls);

    let state = Arc::new(AppState {
        db_pool: pool,
        config: cfg,
        start_time: Instant::now(),
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(state.clone()))
            .app_data(web::JsonConfig::default().limit(10 * 1024 * 1024))
            .wrap(TracingLogger::default())
            .configure(api::configure_routes)
    });

    if let Some(tls) = tls_config {
        tracing::info!("Starting server with TLS on {}", bind_addr);
        server.bind_rustls_0_22(&bind_addr, tls)?.run().await
    } else {
        tracing::info!("Starting server (plain HTTP) on {}", bind_addr);
        server.bind(&bind_addr)?.run().await
    }
}
