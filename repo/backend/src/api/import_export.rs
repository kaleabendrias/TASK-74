use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::StreamExt;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::ExportRequest;
use crate::require_role;
use crate::service::import_export as svc;
use crate::AppState;

/// Uploads an .xlsx file and creates a queued import job.
pub async fn upload_import(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, InventoryClerk);

    let mut file_data: Vec<u8> = Vec::new();
    let mut original_name = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            ApiError::bad_request("MULTIPART_ERROR", &e.to_string())
        })?;

        if field.name() == Some("file") {
            original_name = field
                .content_disposition()
                .get_filename()
                .unwrap_or("import.xlsx")
                .to_string();

            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| {
                    ApiError::bad_request("MULTIPART_ERROR", &e.to_string())
                })?;
                file_data.extend_from_slice(&data);
            }
        }
    }

    if file_data.is_empty() {
        return Err(ApiError::bad_request(
            "MISSING_FILE",
            "No .xlsx file found in upload",
        ));
    }

    // Validate extension
    if !original_name.ends_with(".xlsx") {
        return Err(ApiError::unprocessable(
            "INVALID_FILE_TYPE",
            "Only .xlsx files are accepted for import",
        ));
    }

    // Save to temp location
    let upload_dir = format!("{}/imports", state.config.uploads.storage_path);
    std::fs::create_dir_all(&upload_dir).map_err(|e| {
        tracing::error!(error = %e, "Failed to create import directory");
        ApiError::internal("Failed to store import file")
    })?;

    let file_id = Uuid::new_v4();
    let file_path = format!("{}/{}.xlsx", upload_dir, file_id);
    std::fs::write(&file_path, &file_data).map_err(|e| {
        tracing::error!(error = %e, "Failed to write import file");
        ApiError::internal("Failed to store import file")
    })?;

    let mut conn = state.db_pool.get()?;
    let job = svc::create_import_job(&mut conn, &file_path, "xlsx_import", ctx.user_id)?;

    Ok(HttpResponse::Created().json(job))
}

/// Retrieves the status and progress of an import job.
pub async fn get_job(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let job = svc::get_import_job(&mut conn, path.into_inner())?;
    Ok(HttpResponse::Ok().json(job))
}

/// Creates a new export request that requires approval before download.
pub async fn request_export(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    body: web::Json<ExportRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let approval = svc::create_export_request(&mut conn, &body.export_type, ctx.user_id)?;
    Ok(HttpResponse::Created().json(approval))
}

/// Approves a pending export request with a watermark.
pub async fn approve_export(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);

    let mut conn = state.db_pool.get()?;
    let approval = svc::approve_export(
        &mut conn,
        path.into_inner(),
        ctx.user_id,
        &ctx.username,
    )?;
    Ok(HttpResponse::Ok().json(approval))
}

/// Downloads an approved export as a watermarked JSON file.
pub async fn download_export(
    state: web::Data<Arc<AppState>>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let approval = svc::get_export_approval(&mut conn, path.into_inner())?;

    if approval.status != "approved" {
        return Err(ApiError::forbidden("Export has not been approved yet"));
    }

    // Generate export data with watermark
    let watermark = approval
        .watermark_text
        .as_deref()
        .unwrap_or("no-watermark");

    let export_data = serde_json::json!({
        "export_type": approval.export_type,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "watermark": watermark,
        "data": [],
    });

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"export_{}.json\"", approval.id),
        ))
        .json(export_data))
}
