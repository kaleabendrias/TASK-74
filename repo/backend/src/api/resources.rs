use actix_web::{web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::{CreateResourceRequest, ResourceQuery, UpdateResourceRequest, UserRole};
use crate::require_role;
use crate::service::resources as svc;
use crate::AppState;

pub async fn create(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    body: web::Json<CreateResourceRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

    let mut conn = state.db_pool.get()?;
    let resource = svc::create_resource(&mut conn, &body, ctx.user_id)?;
    Ok(HttpResponse::Created().json(resource))
}

pub async fn get(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let resource = svc::get_resource(&mut conn, path.into_inner())?;
    Ok(HttpResponse::Ok().json(resource))
}

pub async fn list(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    query: web::Query<ResourceQuery>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;

    // Clinicians see only their facility's resources
    let scope = ctx.scope_facility();
    let result = svc::list_resources(&mut conn, &query, scope)?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn update(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
    body: web::Json<UpdateResourceRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher, Reviewer);

    let mut conn = state.db_pool.get()?;
    let resource = svc::update_resource(&mut conn, path.into_inner(), &body, ctx.user_id, ctx.role)?;
    Ok(HttpResponse::Ok().json(resource))
}
