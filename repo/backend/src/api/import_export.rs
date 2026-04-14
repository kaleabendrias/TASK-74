use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::model::ExportRequest;
use crate::require_role;
use crate::service::import_export as svc;
use crate::AppState;

/// Uploads an .xlsx file and creates a queued import job.
pub async fn upload_import(
    state: web::Data<AppState>,
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
                .and_then(|cd| cd.get_filename().map(|s| s.to_string()))
                .unwrap_or_else(|| "import.xlsx".to_string());

            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| {
                    ApiError::bad_request("MULTIPART_ERROR", &e.to_string())
                })?;
                file_data.extend_from_slice(&data);
                // Enforce 50MB upload limit
                if file_data.len() > 52_428_800 {
                    return Err(ApiError::unprocessable(
                        "FILE_TOO_LARGE",
                        "Import file exceeds 50 MB limit",
                    ));
                }
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

    // Verify XLSX signature (PK zip header)
    if file_data.len() < 4 || &file_data[..4] != b"PK\x03\x04" {
        return Err(ApiError::unprocessable(
            "INVALID_FILE_TYPE",
            "File does not appear to be a valid .xlsx (ZIP) archive",
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
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let job = svc::get_import_job(&mut conn, path.into_inner())?;

    // Verify ownership: job creator or Administrator
    if job.created_by != ctx.user_id {
        ctx.require_any_role(&[crate::model::UserRole::Administrator])?;
    }

    Ok(HttpResponse::Ok().json(job))
}

/// Creates a new export request that requires approval before download.
pub async fn request_export(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<ExportRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let approval = svc::create_export_request(&mut conn, &body.export_type, ctx.user_id)?;
    Ok(HttpResponse::Created().json(approval))
}

/// Approves a pending export request with a watermark.
pub async fn approve_export(
    state: web::Data<AppState>,
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
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let approval = svc::get_export_approval(&mut conn, path.into_inner())?;

    // Verify the requester has access
    if approval.requested_by != ctx.user_id {
        ctx.require_any_role(&[crate::model::UserRole::Administrator])?;
    }

    if approval.status != "approved" {
        return Err(ApiError::forbidden("Export has not been approved yet"));
    }

    let watermark = approval.watermark_text.as_deref().unwrap_or("no-watermark");

    // Facility-scoped export: Clinicians and InventoryClerks only see their facility's data
    let facility_clause = match ctx.scope_facility() {
        Some(fid) => format!("WHERE facility_id = '{}'", fid),
        None => String::new(),
    };

    // Query real data based on export_type
    let mut data: serde_json::Value = match approval.export_type.as_str() {
        "resources" => {
            let q = format!("SELECT row_to_json(r) as doc FROM resources r {} ORDER BY created_at DESC LIMIT 10000", facility_clause);
            let rows: Vec<serde_json::Value> = diesel::sql_query(q)
            .load::<crate::repository::JsonRow>(&mut conn)
            .map_err(|e| {
                tracing::error!(error = %e, export_type = %approval.export_type, "Export query failed");
                ApiError::internal("Failed to generate export data")
            })?
            .into_iter().map(|r| r.doc).collect();
            serde_json::json!(rows)
        }
        "lodgings" => {
            let q = format!("SELECT row_to_json(l) as doc FROM lodgings l {} ORDER BY created_at DESC LIMIT 10000", facility_clause);
            let rows: Vec<serde_json::Value> = diesel::sql_query(q)
            .load::<crate::repository::JsonRow>(&mut conn)
            .map_err(|e| {
                tracing::error!(error = %e, export_type = %approval.export_type, "Export query failed");
                ApiError::internal("Failed to generate export data")
            })?
            .into_iter().map(|r| r.doc).collect();
            serde_json::json!(rows)
        }
        "inventory" => {
            let q = format!("SELECT row_to_json(i) as doc FROM inventory_lots i {} ORDER BY created_at DESC LIMIT 10000", facility_clause);
            let rows: Vec<serde_json::Value> = diesel::sql_query(q)
            .load::<crate::repository::JsonRow>(&mut conn)
            .map_err(|e| {
                tracing::error!(error = %e, export_type = %approval.export_type, "Export query failed");
                ApiError::internal("Failed to generate export data")
            })?
            .into_iter().map(|r| r.doc).collect();
            serde_json::json!(rows)
        }
        "transactions" => {
            let q = if facility_clause.is_empty() {
                "SELECT row_to_json(t) as doc FROM inventory_transactions t ORDER BY t.created_at DESC LIMIT 10000".to_string()
            } else {
                format!(
                    "SELECT row_to_json(t) as doc FROM inventory_transactions t \
                     INNER JOIN inventory_lots l ON t.lot_id = l.id \
                     WHERE l.facility_id = '{}' \
                     ORDER BY t.created_at DESC LIMIT 10000",
                    ctx.scope_facility().unwrap()
                )
            };
            let rows: Vec<serde_json::Value> = diesel::sql_query(q)
            .load::<crate::repository::JsonRow>(&mut conn)
            .map_err(|e| {
                tracing::error!(error = %e, export_type = %approval.export_type, "Export query failed");
                ApiError::internal("Failed to generate export data")
            })?
            .into_iter().map(|r| r.doc).collect();
            serde_json::json!(rows)
        }
        _ => serde_json::json!([]),
    };

    mask_pii_fields(&mut data);

    let export_data = serde_json::json!({
        "export_type": approval.export_type,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "watermark": watermark,
        "data": data,
    });

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"export_{}.json\"", approval.id),
        ))
        .json(export_data))
}

/// Lists all pending export approvals for reviewers.
pub async fn list_pending_exports(
    state: web::Data<AppState>,
    ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);
    let mut conn = state.db_pool.get()?;
    let rows = svc::list_pending_exports(&mut conn)?;
    Ok(HttpResponse::Ok().json(rows))
}

/// Masks PII fields in export data rows.
fn mask_pii_fields(data: &mut serde_json::Value) {
    if let Some(arr) = data.as_array_mut() {
        for row in arr.iter_mut() {
            if let Some(obj) = row.as_object_mut() {
                // Mask email-like fields
                for key in &["email", "contact_email", "contact_info"] {
                    if let Some(val) = obj.get_mut(*key) {
                        if let Some(s) = val.as_str() {
                            *val = serde_json::Value::String(crate::service::masking::mask_email(s));
                        }
                    }
                }
                // Mask phone-like fields
                for key in &["phone", "contact_phone", "phone_number"] {
                    if let Some(val) = obj.get_mut(*key) {
                        if let Some(s) = val.as_str() {
                            *val = serde_json::Value::String(crate::service::masking::mask_phone(s));
                        }
                    }
                }
            }
        }
    }
}
