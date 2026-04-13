mod health;
mod auth;
mod resources;
mod lodgings;
mod inventory;
mod media;
mod import_export;
mod connector;
mod metrics;

use actix_web::web;

/// Registers all API route handlers on the given service configuration.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health::health_check))
            // Auth
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/logout", web::post().to(auth::logout))
            .route("/auth/me", web::get().to(auth::me))
            // Resources
            .route("/resources", web::post().to(resources::create))
            .route("/resources", web::get().to(resources::list))
            .route("/resources/{id}", web::get().to(resources::get))
            .route("/resources/{id}", web::put().to(resources::update))
            // Lodgings
            .route("/lodgings", web::post().to(lodgings::create))
            .route("/lodgings", web::get().to(lodgings::list))
            .route("/lodgings/{id}", web::get().to(lodgings::get))
            .route("/lodgings/{id}", web::put().to(lodgings::update))
            .route("/lodgings/{id}/periods", web::get().to(lodgings::get_periods))
            .route("/lodgings/{id}/periods", web::put().to(lodgings::upsert_period))
            .route(
                "/lodgings/{id}/rent-change",
                web::put().to(lodgings::request_rent_change),
            )
            .route(
                "/lodgings/{id}/rent-change/{change_id}/approve",
                web::post().to(lodgings::approve_rent_change),
            )
            .route(
                "/lodgings/{id}/rent-change/{change_id}/reject",
                web::post().to(lodgings::reject_rent_change),
            )
            // Inventory
            .route("/inventory/lots", web::post().to(inventory::create_lot))
            .route("/inventory/lots", web::get().to(inventory::list_lots))
            .route("/inventory/lots/{id}", web::get().to(inventory::get_lot))
            .route(
                "/inventory/lots/{id}/reserve",
                web::post().to(inventory::reserve),
            )
            .route(
                "/inventory/transactions",
                web::post().to(inventory::create_transaction),
            )
            .route(
                "/inventory/transactions",
                web::get().to(inventory::list_transactions),
            )
            .route(
                "/inventory/transactions/audit-print",
                web::get().to(inventory::audit_print),
            )
            // Media
            .route("/media/upload", web::post().to(media::upload))
            .route("/media/{id}/download", web::get().to(media::download))
            // Import / Export
            .route("/import/upload", web::post().to(import_export::upload_import))
            .route("/import/jobs/{id}", web::get().to(import_export::get_job))
            .route(
                "/export/request",
                web::post().to(import_export::request_export),
            )
            .route(
                "/export/approve/{id}",
                web::post().to(import_export::approve_export),
            )
            .route(
                "/export/download/{id}",
                web::get().to(import_export::download_export),
            )
            // Connector
            .route("/connector/inbound", web::post().to(connector::inbound))
            // Metrics
            .route("/metrics", web::get().to(metrics::prometheus_metrics)),
    );
}
