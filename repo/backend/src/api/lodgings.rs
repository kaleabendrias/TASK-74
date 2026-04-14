use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::*;
use crate::repository::lodgings as lodging_repo;
use crate::require_role;
use crate::service::lodgings as svc;
use crate::AppState;

/// Verifies that a facility-scoped user is accessing a lodging that belongs
/// to their facility. Administrators (scope_facility() == None) pass through.
/// Lodgings with no facility_id are accessible to everyone.
fn enforce_lodging_facility(ctx: &RbacContext, lodging_facility_id: Option<Uuid>) -> Result<(), ApiError> {
    if let Some(scoped) = ctx.scope_facility() {
        match lodging_facility_id {
            Some(fid) if fid != scoped => {
                return Err(ApiError::forbidden("Access denied: lodging belongs to a different facility"));
            }
            None => {
                return Err(ApiError::forbidden("Access denied: lodging has no facility assignment"));
            }
            _ => {} // facility matches
        }
    }
    Ok(())
}

/// Creates a new lodging entry (requires Administrator or Publisher role).
pub async fn create(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<CreateLodgingRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let mut conn = state.db_pool.get()?;
    let lodging = svc::create_lodging(&mut conn, &body, ctx.user_id)?;
    Ok(HttpResponse::Created().json(lodging))
}

/// Retrieves a single lodging by its ID.
pub async fn get(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer, Clinician);
    let lodging_id = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let row = lodging_repo::find_lodging_by_id(&mut conn, lodging_id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, row.facility_id)?;

    let lodging = svc::get_lodging(&mut conn, lodging_id)?;
    Ok(HttpResponse::Ok().json(lodging))
}

/// Lists lodgings scoped to the user's facility when applicable.
pub async fn list(
    state: web::Data<AppState>,
    ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer, Clinician);
    let mut conn = state.db_pool.get()?;
    let scope = ctx.scope_facility();
    let lodgings = svc::list_lodgings(&mut conn, scope)?;
    Ok(HttpResponse::Ok().json(lodgings))
}

/// Updates an existing lodging by ID.
pub async fn update(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<UpdateLodgingRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer);

    let lodging_id = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let row = lodging_repo::find_lodging_by_id(&mut conn, lodging_id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, row.facility_id)?;

    let lodging = svc::update_lodging(&mut conn, lodging_id, &body, ctx.role)?;
    Ok(HttpResponse::Ok().json(lodging))
}

// ── Periods ──

/// Returns all availability periods for a lodging.
pub async fn get_periods(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer, Clinician);
    let lodging_id = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let row = lodging_repo::find_lodging_by_id(&mut conn, lodging_id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, row.facility_id)?;

    let periods = svc::get_periods(&mut conn, lodging_id)?;
    Ok(HttpResponse::Ok().json(periods))
}

/// Creates or replaces an availability period for a lodging.
pub async fn upsert_period(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<LodgingPeriodRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let lodging_id = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let row = lodging_repo::find_lodging_by_id(&mut conn, lodging_id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, row.facility_id)?;

    let period = svc::upsert_period(&mut conn, lodging_id, &body)?;
    Ok(HttpResponse::Created().json(period))
}

// ── Rent Changes ──

/// Submits a rent change request for a lodging.
pub async fn request_rent_change(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<RentChangeRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let lodging_id = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let row = lodging_repo::find_lodging_by_id(&mut conn, lodging_id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, row.facility_id)?;

    let change = svc::request_rent_change(&mut conn, lodging_id, &body, ctx.user_id)?;
    Ok(HttpResponse::Created().json(change))
}

/// Lists all pending rent change requests for reviewer action.
pub async fn list_pending_rent_changes(
    state: web::Data<AppState>,
    ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);
    let mut conn = state.db_pool.get()?;
    let rows = svc::list_pending_rent_changes(&mut conn)?;
    Ok(HttpResponse::Ok().json(rows))
}

#[derive(serde::Deserialize)]
pub struct RentChangePath {
    id: Uuid,
    change_id: Uuid,
}

/// Approves a pending rent change request.
pub async fn approve_rent_change(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<RentChangePath>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);

    let p = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let lodging = lodging_repo::find_lodging_by_id(&mut conn, p.id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, lodging.facility_id)?;

    let change = svc::approve_rent_change(&mut conn, p.id, p.change_id, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(change))
}

/// Rejects a pending rent change request.
pub async fn reject_rent_change(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<RentChangePath>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);

    let p = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let lodging = lodging_repo::find_lodging_by_id(&mut conn, p.id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, lodging.facility_id)?;

    let change = svc::reject_rent_change(&mut conn, p.id, p.change_id, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(change))
}

/// Submits a Reviewer's counterproposal on a pending rent change.
/// Transitions the change to status = 'countered', recording the alternative
/// rent and deposit values the publisher should consider.
pub async fn counterpropose_rent_change(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<RentChangePath>,
    body: web::Json<crate::model::CounterproposalRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);

    let p = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let lodging = lodging_repo::find_lodging_by_id(&mut conn, p.id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, lodging.facility_id)?;

    let change = svc::counterpropose_rent_change(&mut conn, p.id, p.change_id, &body, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(change))
}

/// Accepts the reviewer's counterproposal on a countered rent change.
/// Applies the counterproposed rent and deposit to the lodging and
/// transitions the change to status = 'approved'. Restricted to the
/// original requester's role group (Publisher / Administrator).
pub async fn accept_counterproposal(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<RentChangePath>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let p = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let lodging = lodging_repo::find_lodging_by_id(&mut conn, p.id)
        .map_err(|_| ApiError::not_found("Lodging"))?;
    enforce_lodging_facility(&ctx, lodging.facility_id)?;

    let change = svc::accept_counterproposal(&mut conn, p.id, p.change_id, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(change))
}
