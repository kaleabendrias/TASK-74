use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::*;
use crate::repository::inventory as repo;
use crate::require_role;
use crate::service::inventory as svc;
use crate::AppState;

/// Lists warehouses, scoped to the requesting user's facility for non-Admins.
pub async fn list_warehouses(
    state: web::Data<AppState>,
    ctx: RbacContext,
    query: web::Query<WarehouseQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let facility_id = ctx.scope_facility().or(query.facility_id);
    let mut conn = state.db_pool.get()?;
    let rows = repo::list_warehouses(&mut conn, facility_id)?;
    let resp: Vec<WarehouseResponse> = rows
        .into_iter()
        .map(|r| WarehouseResponse { id: r.id, facility_id: r.facility_id, name: r.name })
        .collect();
    Ok(HttpResponse::Ok().json(resp))
}

/// Lists bins for a given warehouse.
pub async fn list_bins(
    state: web::Data<AppState>,
    ctx: RbacContext,
    query: web::Query<BinQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let mut conn = state.db_pool.get()?;
    let rows = repo::list_bins(&mut conn, query.warehouse_id)?;
    let resp: Vec<BinResponse> = rows
        .into_iter()
        .map(|r| BinResponse { id: r.id, warehouse_id: r.warehouse_id, label: r.label })
        .collect();
    Ok(HttpResponse::Ok().json(resp))
}

/// Verifies that a facility-scoped user is accessing an entity that belongs
/// to their facility. Administrators (scope_facility() == None) pass through.
fn enforce_facility(ctx: &RbacContext, entity_facility_id: Uuid) -> Result<(), ApiError> {
    if let Some(scoped) = ctx.scope_facility() {
        if scoped != entity_facility_id {
            return Err(ApiError::forbidden("Access denied: entity belongs to a different facility"));
        }
    }
    Ok(())
}

/// Creates a new inventory lot.
pub async fn create_lot(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<CreateLotRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);
    enforce_facility(&ctx, body.facility_id)?;

    let mut conn = state.db_pool.get()?;
    let lot = svc::create_lot(&mut conn, &body)?;
    Ok(HttpResponse::Created().json(lot))
}

/// Retrieves a single inventory lot by its ID.
pub async fn get_lot(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let mut conn = state.db_pool.get()?;
    let lot = svc::get_lot(&mut conn, path.into_inner())?;
    enforce_facility(&ctx, lot.facility_id)?;
    Ok(HttpResponse::Ok().json(lot))
}

/// Lists inventory lots with optional facility and expiry filtering.
pub async fn list_lots(
    state: web::Data<AppState>,
    ctx: RbacContext,
    query: web::Query<LotQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let mut conn = state.db_pool.get()?;
    // Facility scoping: scoped users always use their own facility; only
    // Administrators may override via the query parameter.
    let facility = ctx.scope_facility().or(query.facility_id);
    let near_expiry = query.near_expiry.unwrap_or(false);
    let lots = svc::list_lots(&mut conn, facility, near_expiry)?;
    Ok(HttpResponse::Ok().json(lots))
}

/// Reserves a quantity from an inventory lot's on-hand stock.
pub async fn reserve(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<ReserveRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let lot_id = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let row = repo::find_lot_by_id(&mut conn, lot_id)
        .map_err(|_| ApiError::not_found("Lot"))?;
    enforce_facility(&ctx, row.facility_id)?;

    let lot = svc::reserve(&mut conn, lot_id, body.quantity, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(lot))
}

/// Records a new inventory transaction (inbound or outbound).
pub async fn create_transaction(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<CreateTransactionRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let mut conn = state.db_pool.get()?;
    let row = repo::find_lot_by_id(&mut conn, body.lot_id)
        .map_err(|_| ApiError::not_found("Lot"))?;
    enforce_facility(&ctx, row.facility_id)?;

    let tx = svc::create_transaction(&mut conn, &body, ctx.user_id)?;
    Ok(HttpResponse::Created().json(tx))
}

/// Lists inventory transactions with optional filtering.
pub async fn list_transactions(
    state: web::Data<AppState>,
    ctx: RbacContext,
    query: web::Query<TransactionQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let mut conn = state.db_pool.get()?;

    // If no lot_id filter, scope transactions to user's facility
    if query.lot_id.is_none() {
        if let Some(fid) = ctx.scope_facility() {
            // For facility-scoped users without a lot_id filter, get all lot IDs
            // in their facility and filter by those
            let facility_lots = repo::list_lots(&mut conn, Some(fid), false)?;
            let lot_ids: Vec<Uuid> = facility_lots.iter().map(|l| l.id).collect();
            // Override the query to filter by these lot IDs
            let txns = svc::list_transactions_for_lots(&mut conn, &lot_ids, &query)?;
            return Ok(HttpResponse::Ok().json(txns));
        }
    } else if let Some(lid) = query.lot_id {
        // Verify the lot belongs to the user's facility
        let lot = repo::find_lot_by_id(&mut conn, lid).map_err(|_| ApiError::not_found("Lot"))?;
        enforce_facility(&ctx, lot.facility_id)?;
    }

    let txns = svc::list_transactions(&mut conn, &query)?;
    Ok(HttpResponse::Ok().json(txns))
}

#[derive(serde::Deserialize)]
pub struct AuditPrintQuery {
    pub lot_id: Uuid,
}

/// Returns an HTML audit trail of transactions for a given lot.
pub async fn audit_print(
    state: web::Data<AppState>,
    ctx: RbacContext,
    query: web::Query<AuditPrintQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let mut conn = state.db_pool.get()?;
    let row = repo::find_lot_by_id(&mut conn, query.lot_id)
        .map_err(|_| ApiError::not_found("Lot"))?;
    enforce_facility(&ctx, row.facility_id)?;

    let html = svc::audit_print_html(&mut conn, query.lot_id)?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
