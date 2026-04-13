use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::Arc;

use crate::errors::ApiError;
use crate::service::connector as svc;
use crate::AppState;

/// Receives and validates a signed inbound connector request.
pub async fn inbound(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, ApiError> {
    // Extract required headers
    let auth_sig = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?;

    let nonce = req
        .headers()
        .get("X-Nonce")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            ApiError::bad_request("MISSING_HEADER", "X-Nonce header is required")
        })?;

    let timestamp = req
        .headers()
        .get("X-Timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            ApiError::bad_request("MISSING_HEADER", "X-Timestamp header is required")
        })?;

    let mut conn = state.db_pool.get()?;
    let ack = svc::validate_and_process(
        &mut conn,
        &state.config.auth.request_signing_key,
        auth_sig,
        &body,
        nonce,
        timestamp,
        "/api/connector/inbound",
    )?;

    Ok(HttpResponse::Ok().json(ack))
}
