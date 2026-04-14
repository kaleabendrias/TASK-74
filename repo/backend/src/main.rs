use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use std::time::Instant;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

use tourism_backend::{api, build_pool, config, jobs, run_migrations, seed_defaults, service, validate_secrets, AppState};

/// Loads rustls server configuration from the certificate and key paths in `cfg`.
///
/// This function is unconditionally required — plain-HTTP startup is not
/// supported in any profile.  Callers **must** provide valid PEM files;
/// the application will panic with a clear diagnostic message otherwise.
fn load_rustls_config(cfg: &config::TlsConfig) -> rustls::ServerConfig {
    let cert_path = &cfg.cert_path;
    let key_path = &cfg.key_path;

    let cert_file = std::fs::File::open(cert_path).unwrap_or_else(|e| {
        panic!(
            "FATAL: Cannot open TLS certificate at '{}': {}. \
             TLS is required in all profiles. Provide a valid certificate.",
            cert_path, e
        )
    });

    let key_file = std::fs::File::open(key_path).unwrap_or_else(|e| {
        panic!(
            "FATAL: Cannot open TLS private key at '{}': {}. \
             TLS is required in all profiles. Provide a valid private key.",
            key_path, e
        )
    });

    let certs: Vec<_> = rustls_pemfile::certs(&mut std::io::BufReader::new(cert_file))
        .filter_map(|r| r.ok())
        .collect();

    if certs.is_empty() {
        panic!(
            "FATAL: No PEM certificates found in '{}'. \
             The file must contain at least one valid certificate block.",
            cert_path
        );
    }

    let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(key_file))
        .unwrap_or_else(|e| {
            panic!(
                "FATAL: Failed to parse TLS private key at '{}': {}",
                key_path, e
            )
        })
        .unwrap_or_else(|| {
            panic!(
                "FATAL: No private key block found in '{}'. \
                 The file must contain a PKCS#8 or RSA private key in PEM format.",
                key_path
            )
        });

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap_or_else(|e| panic!("FATAL: Failed to build TLS ServerConfig: {}", e))
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
    validate_secrets(&cfg);
    let pool = build_pool(&cfg.database.url, cfg.database.max_connections);

    run_migrations(&pool);
    seed_defaults(&pool);
    jobs::spawn_job_runner(pool.clone());
    jobs::spawn_scheduled_publisher(pool.clone());

    // Optionally start the on-prem MQ connector (AMQP or TCP transport)
    if cfg.mq.enabled {
        let mq_connector = Arc::new(service::mq_connector::HmacMqConnector::new(
            pool.clone(),
            cfg.auth.request_signing_key.clone(),
        ));
        if let Some(ref amqp_url) = cfg.mq.amqp_url {
            service::mq_connector::spawn_amqp_consumer(
                amqp_url.clone(),
                cfg.mq.amqp_queue.clone(),
                mq_connector,
            );
        } else {
            service::mq_connector::spawn_mq_listener(cfg.mq.bind_address.clone(), mq_connector);
        }
    }

    let bind_addr = format!("{}:{}", cfg.server.bind_address, cfg.server.bind_port);

    // TLS is mandatory across all profiles.  load_rustls_config panics rather
    // than falling back to plain HTTP, so there is no code path that starts
    // the server without encryption.
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

    tracing::info!("Starting server with TLS on {}", bind_addr);
    server.bind_rustls_0_22(&bind_addr, tls_config)?.run().await
}
