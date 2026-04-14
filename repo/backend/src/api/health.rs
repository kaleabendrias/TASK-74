use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use diesel::sql_query;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::{HealthResponse, LivenessResponse};
use crate::AppState;

/// Public liveness probe for load balancers and orchestrators.
/// Returns `{"status":"ok"}` (or `"degraded"` if the DB is unreachable).
/// No authentication required — intentionally minimal to avoid leaking
/// internal configuration details to unauthenticated callers.
pub async fn liveness(state: web::Data<AppState>) -> HttpResponse {
    let db_ok = state
        .db_pool
        .get()
        .ok()
        .and_then(|mut c| sql_query("SELECT 1").execute(&mut c).ok())
        .is_some();

    let status = if db_ok { "ok" } else { "degraded" };
    HttpResponse::Ok().json(LivenessResponse { status: status.into() })
}

/// Protected readiness probe — requires authentication.
/// Returns the full service details: version, config profile, uptime, and
/// disk usage. Kept behind auth to prevent leaking operational metadata to
/// anonymous clients or external scanners.
pub async fn readiness(
    state: web::Data<AppState>,
    _ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    let db_connected = match state.db_pool.get() {
        Ok(mut conn) => sql_query("SELECT 1").execute(&mut conn).is_ok(),
        Err(_) => false,
    };

    let response = HealthResponse {
        service: state.config.app.service_name.clone(),
        version: state.config.app.version.clone(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        database_connected: db_connected,
        disk_usage_bytes: get_disk_usage(),
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
