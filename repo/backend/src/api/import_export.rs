use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures_util::StreamExt;
use rust_xlsxwriter::{Format, Workbook};
use std::time::Duration;
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
                    return Err(ApiError::payload_too_large(
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
/// Restricted to Administrators and Reviewers to prevent approval-queue spam
/// and unauthorized data extraction.
pub async fn request_export(
    state: web::Data<AppState>,
    ctx: RbacContext,
    body: web::Json<ExportRequest>,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Reviewer);

    let mut conn = state.db_pool.get()?;
    let approval = svc::create_export_request(&mut conn, &body.export_type, ctx.user_id)?;
    Ok(HttpResponse::Created().json(approval))
}

/// Approves a pending export request, optionally stamping a watermark when the
/// `export_watermark` feature flag is enabled.
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
        state.config.features.export_watermark,
    )?;
    Ok(HttpResponse::Ok().json(approval))
}

/// Downloads an approved export as a watermarked `.xlsx` Excel file.
///
/// The workbook contains two sheets:
///   - **"Metadata"** — export_type, generated_at, watermark, and the approval ID.
///   - **"Data"** — one header row (column names) followed by data rows.
///
/// PII fields (email, phone) are masked in the data sheet before writing.
/// Approval and watermarking logic is fully preserved.
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

    // ── Build the .xlsx workbook ──────────────────────────────────────────
    let xlsx_bytes = build_xlsx_export(&approval.export_type, watermark, &approval.id, &data)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to build xlsx workbook");
            ApiError::internal("Failed to generate Excel export")
        })?;

    Ok(HttpResponse::Ok()
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"export_{}.xlsx\"", approval.id),
        ))
        .body(xlsx_bytes))
}

/// Serialises `data` (a JSON array of objects) into an `.xlsx` workbook and
/// returns the raw bytes.
///
/// Sheet layout:
///   - Sheet 1 "Metadata": key/value pairs — export type, watermark, generated timestamp
///   - Sheet 2 "Data": header row + one row per JSON object; cells are written
///     as strings so no type-inference surprises occur for IDs or ISO timestamps.
fn build_xlsx_export(
    export_type: &str,
    watermark: &str,
    export_id: &Uuid,
    data: &serde_json::Value,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();

    // ── Sheet 1: Metadata ──
    let bold = Format::new().set_bold();
    {
        let meta = workbook.add_worksheet();
        meta.set_name("Metadata")?;
        meta.write_with_format(0, 0, "Export Type", &bold)?;
        meta.write(0, 1, export_type)?;
        meta.write_with_format(1, 0, "Export ID", &bold)?;
        meta.write(1, 1, export_id.to_string().as_str())?;
        meta.write_with_format(2, 0, "Generated At", &bold)?;
        meta.write(2, 1, chrono::Utc::now().to_rfc3339().as_str())?;
        meta.write_with_format(3, 0, "Watermark", &bold)?;
        meta.write(3, 1, watermark)?;
        meta.set_column_width(0, 18)?;
        meta.set_column_width(1, 42)?;
    }

    // ── Sheet 2: Data ──
    {
        let rows = data.as_array().map(|a| a.as_slice()).unwrap_or_default();
        let data_sheet = workbook.add_worksheet();
        data_sheet.set_name("Data")?;

        if rows.is_empty() {
            data_sheet.write(0, 0, "(no data)")?;
        } else {
            // Collect column names from the first object's keys (stable insertion order)
            let columns: Vec<String> = if let Some(obj) = rows[0].as_object() {
                obj.keys().cloned().collect()
            } else {
                vec![]
            };

            // Header row
            for (col_idx, col_name) in columns.iter().enumerate() {
                data_sheet.write_with_format(0, col_idx as u16, col_name.as_str(), &bold)?;
            }

            // Data rows
            for (row_idx, row) in rows.iter().enumerate() {
                let excel_row = (row_idx + 1) as u32;
                if let Some(obj) = row.as_object() {
                    for (col_idx, col_name) in columns.iter().enumerate() {
                        let cell_val = match obj.get(col_name) {
                            Some(serde_json::Value::String(s)) => s.clone(),
                            Some(serde_json::Value::Number(n)) => n.to_string(),
                            Some(serde_json::Value::Bool(b)) => b.to_string(),
                            Some(serde_json::Value::Null) | None => String::new(),
                            Some(v) => v.to_string(),
                        };
                        data_sheet.write(excel_row, col_idx as u16, cell_val.as_str())?;
                    }
                }
            }

            // Auto-width approximation for readability
            for col_idx in 0..columns.len() {
                data_sheet.set_column_width(col_idx as u16, 20)?;
            }
        }
    }

    Ok(workbook.save_to_buffer()?)
}

/// Streams import job status as Server-Sent Events (SSE).
///
/// Sends a `data:` event containing the `ImportJobResponse` JSON every 750 ms.
/// The stream terminates automatically once the job reaches `completed` or `failed`.
/// The client should close the `EventSource` on receipt of a terminal event.
pub async fn stream_job_status(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let job_id = path.into_inner();

    // Verify ownership before opening the stream
    let mut conn = state.db_pool.get()?;
    let initial = svc::get_import_job(&mut conn, job_id)?;
    if initial.created_by != ctx.user_id {
        ctx.require_any_role(&[crate::model::UserRole::Administrator])?;
    }
    drop(conn);

    let pool = state.db_pool.clone();

    let sse_stream = futures_util::stream::unfold(
        (pool, job_id, false),
        |(pool, id, done)| async move {
            if done {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(750)).await;

            let pool_c = pool.clone();
            let job_opt = tokio::task::spawn_blocking(move || {
                let mut conn = pool_c.get().ok()?;
                crate::service::import_export::get_import_job(&mut *conn, id).ok()
            })
            .await
            .ok()
            .flatten();

            match job_opt {
                Some(job) => {
                    let terminal = job.status == "completed" || job.status == "failed";
                    let data = serde_json::to_string(&job).unwrap_or_default();
                    let event = web::Bytes::from(format!("data: {}\n\n", data));
                    Some((Ok::<_, actix_web::Error>(event), (pool, id, terminal)))
                }
                None => None,
            }
        },
    );

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(sse_stream))
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
