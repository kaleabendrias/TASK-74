use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use diesel::sql_query;

use crate::errors::ApiError;
use crate::model::HealthResponse;
use crate::AppState;

/// Returns the service health status including database connectivity and uptime.
pub async fn health_check(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let db_connected = match state.db_pool.get() {
        Ok(mut conn) => sql_query("SELECT 1").execute(&mut conn).is_ok(),
        Err(_) => false,
    };

    let disk_usage = get_disk_usage();

    let response = HealthResponse {
        service: state.config.app.service_name.clone(),
        version: state.config.app.version.clone(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        database_connected: db_connected,
        disk_usage_bytes: disk_usage,
        config_profile: state.config.app.config_profile.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}

fn get_disk_usage() -> Option<u64> {
    let output = std::process::Command::new("df")
        .args(["--output=used", "-B1", "/"])
        .output()
        .ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout.lines().nth(1)?.trim().parse::<u64>().ok()
}
