use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::{CreateResourceRequest, ResourceQuery, UpdateResourceRequest};
use crate::require_role;
use crate::service::resources as svc;
use crate::AppState;

/// Creates a new resource entry (requires Administrator or Publisher role).
pub async fn create(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<CreateResourceRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let facility_id = ctx.facility_id; // Assign creator's facility

    let mut conn = state.db_pool.get()?;
    let resource = svc::create_resource(&mut conn, &body, ctx.user_id, &state.config.crypto.aes256_master_key, facility_id)?;
    Ok(HttpResponse::Created().json(resource))
}

/// Retrieves a single resource by its ID.
pub async fn get(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer, Clinician);

    let mut conn = state.db_pool.get()?;
    let resource = svc::get_resource(&mut conn, path.into_inner())?;

    // Enforce facility scoping if the user is facility-scoped
    if let Some(fid) = ctx.scope_facility() {
        match resource.facility_id {
            Some(rfid) if rfid != fid => {
                return Err(ApiError::forbidden("Access denied: resource belongs to a different facility"));
            }
            None => {
                return Err(ApiError::forbidden("Access denied: resource has no facility assignment"));
            }
            _ => {}
        }
    }

    Ok(HttpResponse::Ok().json(resource))
}

/// Lists resources with optional filtering and facility-scoped access.
pub async fn list(
    state: web::Data<AppState>,
    ctx: RbacContext,
    query: web::Query<ResourceQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer, Clinician);

    let mut conn = state.db_pool.get()?;

    // Clinicians see only their facility's resources
    let scope = ctx.scope_facility();
    let result = svc::list_resources(&mut conn, &query, scope)?;
    Ok(HttpResponse::Ok().json(result))
}

/// Returns the version history of a resource.
pub async fn list_versions(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer);

    let resource_id = path.into_inner();
    let mut conn = state.db_pool.get()?;

    // Load resource and enforce facility ownership
    let existing = svc::get_resource(&mut conn, resource_id)?;
    if let Some(fid) = ctx.scope_facility() {
        match existing.facility_id {
            Some(rfid) if rfid != fid => {
                return Err(ApiError::forbidden("Access denied: resource belongs to a different facility"));
            }
            None => {
                return Err(ApiError::forbidden("Access denied: resource has no facility assignment"));
            }
            _ => {}
        }
    }

    let versions = svc::list_versions(&mut conn, resource_id)?;
    Ok(HttpResponse::Ok().json(versions))
}

/// Updates an existing resource by ID.
pub async fn update(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<UpdateResourceRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer);

    let resource_id = path.into_inner();
    let mut conn = state.db_pool.get()?;

    // Load resource and enforce facility ownership
    let existing = svc::get_resource(&mut conn, resource_id)?;
    if let Some(fid) = ctx.scope_facility() {
        match existing.facility_id {
            Some(rfid) if rfid != fid => {
                return Err(ApiError::forbidden("Access denied: resource belongs to a different facility"));
            }
            None => {
                return Err(ApiError::forbidden("Access denied: resource has no facility assignment"));
            }
            _ => {}
        }
    }

    let resource = svc::update_resource(&mut conn, resource_id, &body, ctx.user_id, ctx.role, &state.config.crypto.aes256_master_key)?;
    Ok(HttpResponse::Ok().json(resource))
}
