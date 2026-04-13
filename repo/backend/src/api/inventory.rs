use actix_web::{web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::*;
use crate::require_role;
use crate::service::inventory as svc;
use crate::AppState;

pub async fn create_lot(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    body: web::Json<CreateLotRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let mut conn = state.db_pool.get()?;
    let lot = svc::create_lot(&mut conn, &body)?;
    Ok(HttpResponse::Created().json(lot))
}

pub async fn get_lot(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let mut conn = state.db_pool.get()?;
    let lot = svc::get_lot(&mut conn, path.into_inner())?;
    Ok(HttpResponse::Ok().json(lot))
}

pub async fn list_lots(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    query: web::Query<LotQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let mut conn = state.db_pool.get()?;
    // Facility scoping for Clinician/InventoryClerk
    let facility = query.facility_id.or(ctx.scope_facility());
    let near_expiry = query.near_expiry.unwrap_or(false);
    let lots = svc::list_lots(&mut conn, facility, near_expiry)?;
    Ok(HttpResponse::Ok().json(lots))
}

pub async fn reserve(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<ReserveRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let mut conn = state.db_pool.get()?;
    let lot = svc::reserve(&mut conn, path.into_inner(), body.quantity, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(lot))
}

pub async fn create_transaction(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    body: web::Json<CreateTransactionRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let mut conn = state.db_pool.get()?;
    let tx = svc::create_transaction(&mut conn, &body, ctx.user_id)?;
    Ok(HttpResponse::Created().json(tx))
}

pub async fn list_transactions(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    query: web::Query<TransactionQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk, Clinician);

    let mut conn = state.db_pool.get()?;
    let txns = svc::list_transactions(&mut conn, &query)?;
    Ok(HttpResponse::Ok().json(txns))
}

#[derive(serde::Deserialize)]
pub struct AuditPrintQuery {
    pub lot_id: Uuid,
}

pub async fn audit_print(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    query: web::Query<AuditPrintQuery>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let mut conn = state.db_pool.get()?;
    let html = svc::audit_print_html(&mut conn, query.lot_id)?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
