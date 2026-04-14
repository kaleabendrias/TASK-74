use actix_web::{web, HttpResponse};

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::repository::config as repo;
use crate::require_role;
use crate::AppState;

/// Lists all configuration parameters for the current profile.
pub async fn list_config(
    state: web::Data<AppState>,
    ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator);

    let mut conn = state.db_pool.get()?;
    let profile = &state.config.app.config_profile;
    let params = repo::list_by_profile(&mut conn, profile)?;
    Ok(HttpResponse::Ok().json(params))
}

#[derive(serde::Deserialize)]
pub struct UpsertConfigRequest {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub feature_switch: bool,
}

/// Reads a single config parameter value by key for runtime feature evaluation.
pub async fn get_config_value(
    state: web::Data<AppState>,
    _ctx: RbacContext,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let profile = &state.config.app.config_profile;
    let params = repo::list_by_profile(&mut conn, profile)?;
    let key = path.into_inner();
    match params.iter().find(|p| p.key == key) {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Err(ApiError::not_found("Config parameter")),
    }
}

/// Creates or updates a configuration parameter.
pub async fn upsert_config(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<UpsertConfigRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator);

    let mut conn = state.db_pool.get()?;
    let profile = &state.config.app.config_profile;
    let row = repo::upsert(&mut conn, profile, &body.key, &body.value, body.feature_switch)?;
    Ok(HttpResponse::Ok().json(row))
}
