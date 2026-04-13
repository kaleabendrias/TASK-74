use actix_web::{web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::*;
use crate::require_role;
use crate::service::lodgings as svc;
use crate::AppState;

pub async fn create(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    body: web::Json<CreateLodgingRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let mut conn = state.db_pool.get()?;
    let lodging = svc::create_lodging(&mut conn, &body, ctx.user_id)?;
    Ok(HttpResponse::Created().json(lodging))
}

pub async fn get(
    state: web::Data<Arc<AppState>>,
    _ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let lodging = svc::get_lodging(&mut conn, path.into_inner())?;
    Ok(HttpResponse::Ok().json(lodging))
}

pub async fn list(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let scope = ctx.scope_facility();
    let lodgings = svc::list_lodgings(&mut conn, scope)?;
    Ok(HttpResponse::Ok().json(lodgings))
}

pub async fn update(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<UpdateLodgingRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer);

    let mut conn = state.db_pool.get()?;
    let lodging = svc::update_lodging(&mut conn, path.into_inner(), &body, ctx.role)?;
    Ok(HttpResponse::Ok().json(lodging))
}

// ── Periods ──

pub async fn get_periods(
    state: web::Data<Arc<AppState>>,
    _ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let periods = svc::get_periods(&mut conn, path.into_inner())?;
    Ok(HttpResponse::Ok().json(periods))
}

pub async fn upsert_period(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<LodgingPeriodRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let mut conn = state.db_pool.get()?;
    let period = svc::upsert_period(&mut conn, path.into_inner(), &body)?;
    Ok(HttpResponse::Created().json(period))
}

// ── Rent Changes ──

pub async fn request_rent_change(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<RentChangeRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let mut conn = state.db_pool.get()?;
    let change = svc::request_rent_change(&mut conn, path.into_inner(), &body, ctx.user_id)?;
    Ok(HttpResponse::Created().json(change))
}

#[derive(serde::Deserialize)]
pub struct RentChangePath {
    id: Uuid,
    change_id: Uuid,
}

pub async fn approve_rent_change(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<RentChangePath>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);

    let p = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let change = svc::approve_rent_change(&mut conn, p.id, p.change_id, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(change))
}

pub async fn reject_rent_change(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<RentChangePath>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);

    let p = path.into_inner();
    let mut conn = state.db_pool.get()?;
    let change = svc::reject_rent_change(&mut conn, p.id, p.change_id, ctx.user_id)?;
    Ok(HttpResponse::Ok().json(change))
}
