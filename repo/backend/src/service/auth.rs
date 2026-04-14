use chrono::{Duration, Utc};
use diesel::PgConnection;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::crypto::{argon2id, csrf, hmac_sign, totp};
use crate::errors::ApiError;
use crate::repository::{sessions, users};

pub struct AuthenticatedSession {
    pub session_token: String,
    pub csrf_token: String,
}

/// Authenticates a user by username/password, optionally verifying TOTP, and creates a session.
pub fn login(
    conn: &mut PgConnection,
    config: &AppConfig,
    username: &str,
    password: &str,
    totp_code: Option<&str>,
) -> Result<AuthenticatedSession, ApiError> {
    let user = users::find_by_username(conn, username)
        .map_err(|_| ApiError::unauthorized("Invalid credentials"))?;

    if !argon2id::verify(password, &user.password_hash) {
        return Err(ApiError::unauthorized("Invalid credentials"));
    }

    // MFA check
    if user.mfa_enabled && config.features.mfa_enabled {
        match totp_code {
            None => {
                // Return a challenge response, not an error
                return Err(ApiError::mfa_challenge());
            }
            Some(code) => {
                if let Some(ref secret) = user.totp_secret {
                    if !totp::verify(secret, code, &config.totp, &config.crypto.aes256_master_key) {
                        return Err(ApiError::unauthorized("Invalid TOTP code"));
                    }
                } else {
                    return Err(ApiError::internal("User MFA enabled but no TOTP secret configured"));
                }
            }
        }
    }

    // Create session
    let session_token = csrf::generate_token();
    let token_hash = hmac_sign::sign(&config.auth.hmac_secret, &session_token);
    let expires_at = Utc::now() + Duration::seconds(config.auth.session_ttl_secs as i64);

    let session = sessions::create_session(
        conn,
        &sessions::NewSession {
            user_id: user.id,
            token_hash: &token_hash,
            expires_at,
        },
    )?;

    // Create CSRF token
    let csrf_token = csrf::generate_token();
    let csrf_hash = hmac_sign::sign(&config.auth.hmac_secret, &csrf_token);
    let csrf_expires = Utc::now() + Duration::seconds(config.auth.csrf_token_ttl_secs as i64);

    sessions::create_csrf_token(
        conn,
        &sessions::NewCsrfToken {
            session_id: session.id,
            token_hash: &csrf_hash,
            expires_at: csrf_expires,
        },
    )?;

    crate::service::audit::log_action(conn, user.id, "login", "session", Some(session.id), None, None);

    Ok(AuthenticatedSession {
        session_token,
        csrf_token,
    })
}

/// Validates a session token and returns `(user_id, session_id)`.
pub fn validate_session(
    conn: &mut PgConnection,
    config: &AppConfig,
    token: &str,
) -> Result<(Uuid, Uuid), ApiError> {
    let token_hash = hmac_sign::sign(&config.auth.hmac_secret, token);
    let session = sessions::find_session_by_token_hash(conn, &token_hash)
        .map_err(|_| ApiError::unauthorized("Invalid or expired session"))?;
    Ok((session.user_id, session.id))
}

/// Invalidates the session associated with the given token.
pub fn logout(
    conn: &mut PgConnection,
    config: &AppConfig,
    token: &str,
) -> Result<(), ApiError> {
    let token_hash = hmac_sign::sign(&config.auth.hmac_secret, token);
    sessions::delete_session_by_token_hash(conn, &token_hash)?;
    Ok(())
}

/// Retrieves a user's profile by their ID.
pub fn get_user_profile(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<users::UserRow, ApiError> {
    users::find_by_id(conn, user_id).map_err(|_| ApiError::not_found("User"))
}
