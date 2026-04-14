use actix_web::{cookie, web, HttpRequest, HttpResponse};
use base64::Engine as _;
use diesel::prelude::*;
use time::Duration as TimeDuration;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::{LoginRequest, LoginResponse, UserProfile};
use crate::service::auth as auth_service;
use crate::AppState;

/// Authenticates a user and returns a session cookie with a CSRF token.
/// When MFA is required but no TOTP code was supplied, returns HTTP 200 with
/// `{"mfa_required": true, "csrf_token": ""}` so the frontend can show the
/// TOTP field without resorting to fragile error-message string matching.
pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;

    let result = match auth_service::login(
        &mut conn,
        &state.config,
        &body.username,
        &body.password,
        body.totp_code.as_deref(),
    ) {
        Ok(session) => session,
        Err(ref e) if e.body.code == "MFA_REQUIRED" => {
            // Return a deterministic 200 challenge payload so the frontend
            // can branch on the structured field rather than parsing the
            // error message string.
            return Ok(HttpResponse::Ok().json(LoginResponse {
                csrf_token: String::new(),
                mfa_required: Some(true),
            }));
        }
        Err(e) => return Err(e),
    };

    // Only set Secure flag if TLS is active (cert_path != /dev/null)
    let tls_active = state.config.tls.cert_path != "/dev/null";

    let session_cookie = cookie::Cookie::build("session", &result.session_token)
        .path("/")
        .http_only(true)
        .secure(tls_active)
        .same_site(cookie::SameSite::Strict)
        .max_age(TimeDuration::seconds(
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
    state: web::Data<AppState>,
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
    let tls_active = state.config.tls.cert_path != "/dev/null";
    let removal = cookie::Cookie::build("session", "")
        .path("/")
        .http_only(true)
        .secure(tls_active)
        .same_site(cookie::SameSite::Strict)
        .max_age(TimeDuration::ZERO)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(removal)
        .json(serde_json::json!({"message": "Logged out"})))
}

/// Returns the profile of the currently authenticated user.
pub async fn me(
    state: web::Data<AppState>,
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

/// Generates a new TOTP secret for the current user to begin MFA enrollment.
pub async fn mfa_setup(
    state: web::Data<AppState>,
    _ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    let secret = crate::crypto::totp::generate_secret();
    let encoded = base64::engine::general_purpose::STANDARD.encode(&secret);
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "secret_base64": encoded,
        "issuer": state.config.totp.issuer,
        "digits": state.config.totp.digits,
        "period": state.config.totp.period_secs,
    })))
}

/// Verifies a TOTP code and enables MFA for the current user.
pub async fn mfa_confirm(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    let secret_b64 = body["secret_base64"].as_str()
        .ok_or_else(|| ApiError::bad_request("MISSING_FIELD", "secret_base64 required"))?;
    let code = body["code"].as_str()
        .ok_or_else(|| ApiError::bad_request("MISSING_FIELD", "TOTP code required"))?;

    let secret_bytes = base64::engine::general_purpose::STANDARD.decode(secret_b64)
        .map_err(|_| ApiError::bad_request("INVALID_SECRET", "Invalid base64 secret"))?;

    // Verify the code against the provided secret
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        state.config.totp.digits as usize,
        1,
        state.config.totp.period_secs as u64,
        secret_bytes.clone(),
    ).map_err(|_| ApiError::internal("Failed to create TOTP verifier"))?;

    if !totp.check_current(code).unwrap_or(false) {
        return Err(ApiError::unauthorized("Invalid TOTP code"));
    }

    // Encrypt the secret and store it
    let encrypted = crate::crypto::aes_gcm::encrypt(&secret_bytes, &state.config.crypto.aes256_master_key);
    let mut conn = state.db_pool.get()?;
    diesel::sql_query(
        "UPDATE users SET totp_secret = $1, mfa_enabled = true, updated_at = now() WHERE id = $2"
    )
    .bind::<diesel::sql_types::Bytea, _>(&encrypted)
    .bind::<diesel::sql_types::Uuid, _>(ctx.user_id)
    .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"mfa_enabled": true})))
}

/// Disables MFA for the current user (requires current TOTP code for security).
pub async fn mfa_disable(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    let code = body["code"].as_str()
        .ok_or_else(|| ApiError::bad_request("MISSING_FIELD", "Current TOTP code required to disable MFA"))?;

    let mut conn = state.db_pool.get()?;
    let user = crate::repository::users::find_by_id(&mut conn, ctx.user_id)?;

    if let Some(ref secret) = user.totp_secret {
        if !crate::crypto::totp::verify(secret, code, &state.config.totp, &state.config.crypto.aes256_master_key) {
            return Err(ApiError::unauthorized("Invalid TOTP code"));
        }
    } else {
        return Err(ApiError::bad_request("MFA_NOT_ENABLED", "MFA is not currently enabled"));
    }

    diesel::sql_query(
        "UPDATE users SET totp_secret = NULL, mfa_enabled = false, updated_at = now() WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(ctx.user_id)
    .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"mfa_enabled": false})))
}
