use actix_web::{cookie, web, HttpRequest, HttpResponse};
use std::sync::Arc;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::{LoginRequest, LoginResponse, UserProfile};
use crate::service::auth as auth_service;
use crate::AppState;

/// Authenticates a user and returns a session cookie with a CSRF token.
pub async fn login(
    state: web::Data<Arc<AppState>>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;

    let result = auth_service::login(
        &mut conn,
        &state.config,
        &body.username,
        &body.password,
        body.totp_code.as_deref(),
    )?;

    // Build HttpOnly Secure SameSite=Strict cookie
    let session_cookie = cookie::Cookie::build("session", &result.session_token)
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(cookie::SameSite::Strict)
        .max_age(cookie::time::Duration::seconds(
            state.config.auth.session_ttl_secs as i64,
        ))
        .finish();

    let response = LoginResponse {
        csrf_token: result.csrf_token,
        mfa_required: None,
    };

    Ok(HttpResponse::Ok()
        .cookie(session_cookie)
        .json(response))
}

/// Logs out the current user by invalidating the session and clearing the cookie.
pub async fn logout(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let token = req
        .cookie("session")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })
        .ok_or_else(|| ApiError::unauthorized("No session"))?;

    let mut conn = state.db_pool.get()?;
    auth_service::logout(&mut conn, &state.config, &token)?;

    // Clear the cookie
    let removal = cookie::Cookie::build("session", "")
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(cookie::SameSite::Strict)
        .max_age(cookie::time::Duration::ZERO)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(removal)
        .json(serde_json::json!({"message": "Logged out"})))
}

/// Returns the profile of the currently authenticated user.
pub async fn me(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let user = auth_service::get_user_profile(&mut conn, ctx.user_id)?;
    let role = crate::model::UserRole::from_str(&user.role)
        .unwrap_or(crate::model::UserRole::Reviewer);

    let profile = UserProfile {
        id: user.id,
        username: user.username,
        role,
        facility_id: user.facility_id,
        mfa_enabled: user.mfa_enabled,
        created_at: user.created_at,
    };
    Ok(HttpResponse::Ok().json(profile))
}
