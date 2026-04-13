use actix_web::{dev::Payload, http::Method, web, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::crypto::hmac_sign;
use crate::errors::ApiError;
use crate::model::UserRole;
use crate::repository::{sessions, users};
use crate::service::auth;
use crate::AppState;

/// Extractor that reads the session cookie, validates it, loads the user,
/// and provides an `RbacContext` to every handler.
#[derive(Debug, Clone)]
pub struct RbacContext {
    pub user_id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub facility_id: Option<Uuid>,
}

impl RbacContext {
    /// Returns the facility_id for data-scope filtering.
    /// Clinicians and InventoryClerks are scoped to their facility;
    /// Administrators see all.
    pub fn scope_facility(&self) -> Option<Uuid> {
        match self.role {
            UserRole::Clinician | UserRole::InventoryClerk => self.facility_id,
            _ => None,
        }
    }

    /// Checks that the user holds one of the specified roles, returning a forbidden error otherwise.
    pub fn require_any_role(&self, roles: &[UserRole]) -> Result<(), ApiError> {
        if roles.contains(&self.role) {
            Ok(())
        } else {
            Err(ApiError::forbidden(&format!(
                "Role {:?} is not authorized for this operation. Required: {:?}",
                self.role, roles
            )))
        }
    }
}

impl FromRequest for RbacContext {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let result = extract_rbac(req);
        ready(result.map_err(|e| e.into()))
    }
}

/// Paths that are exempt from CSRF validation.
const CSRF_EXEMPT_PATHS: &[&str] = &["/api/auth/login", "/api/connector/inbound"];

/// Returns `true` for HTTP methods that do not mutate state and therefore
/// do not require a CSRF token.
fn is_safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

/// Validates the CSRF token from the `X-CSRF-Token` header against the
/// `csrf_tokens` table for the given session.
fn validate_csrf(
    req: &HttpRequest,
    conn: &mut diesel::PgConnection,
    hmac_secret: &str,
    session_id: Uuid,
) -> Result<(), ApiError> {
    let csrf_raw = req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::forbidden("Missing CSRF token"))?;

    let csrf_hash = hmac_sign::sign(hmac_secret, csrf_raw);

    sessions::find_csrf_token(conn, &csrf_hash, session_id)
        .map_err(|_| ApiError::forbidden("Invalid or expired CSRF token"))?;

    Ok(())
}

fn extract_rbac(req: &HttpRequest) -> Result<RbacContext, ApiError> {
    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ApiError::internal("App state not configured"))?;

    // Read session token from cookie
    let token = req
        .cookie("session")
        .map(|c| c.value().to_string())
        .or_else(|| {
            // Fallback: Authorization: Bearer <token>
            req.headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })
        .ok_or_else(|| ApiError::unauthorized("No session cookie or Authorization header"))?;

    let mut conn = state.db_pool.get()?;

    let (user_id, session_id) = auth::validate_session(&mut conn, &state.config, &token)?;

    // CSRF validation for mutating requests on non-exempt paths
    if !is_safe_method(req.method()) {
        let path = req.path();
        let exempt = CSRF_EXEMPT_PATHS.iter().any(|p| path == *p);
        if !exempt {
            validate_csrf(req, &mut conn, &state.config.auth.hmac_secret, session_id)?;
        }
    }

    let user = users::find_by_id(&mut conn, user_id)
        .map_err(|_| ApiError::unauthorized("User not found"))?;

    let role = UserRole::from_str(&user.role)
        .ok_or_else(|| ApiError::internal(&format!("Unknown role: {}", user.role)))?;

    Ok(RbacContext {
        user_id: user.id,
        username: user.username,
        role,
        facility_id: user.facility_id,
    })
}

// The require_role! macro is defined in lib.rs for proper crate-level export.
