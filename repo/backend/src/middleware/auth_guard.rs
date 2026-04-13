use actix_web::{dev::Payload, web, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::errors::ApiError;
use crate::model::UserRole;
use crate::repository::users;
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

    let user_id = auth::validate_session(&mut conn, &state.config, &token)?;
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
