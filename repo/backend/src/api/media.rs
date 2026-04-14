use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::require_role;
use crate::service::media as svc;
use crate::AppState;

/// Handles multipart file upload and stores the media file.
pub async fn upload(
    state: web::Data<AppState>,
    ctx: RbacContext,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    require_role!(ctx, Administrator, Publisher);

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
                .unwrap_or_else(|| "unknown".to_string());

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
            "No file found in multipart upload. Use field name 'file'.",
        ));
    }

    let mut conn = state.db_pool.get()?;
    let result = svc::process_upload(
        &mut conn,
        &state.config.uploads,
        &original_name,
        &file_data,
        ctx.user_id,
    )?;

    Ok(HttpResponse::Created().json(result))
}

/// Downloads a media file by its ID.
pub async fn download(
    state: web::Data<AppState>,
    ctx: RbacContext,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;
    let (meta, data) = svc::get_file(&mut conn, path.into_inner())?;

    // Enforce facility scope: check if uploader is in the same facility
    if let Some(scoped_fid) = ctx.scope_facility() {
        let uploader = crate::repository::users::find_by_id(&mut conn, meta.uploaded_by)
            .map_err(|_| ApiError::internal("Uploader not found"))?;
        if uploader.facility_id != Some(scoped_fid) {
            return Err(ApiError::forbidden("Access denied: media belongs to a different facility"));
        }
    }

    Ok(HttpResponse::Ok()
        .content_type(meta.mime_type.as_str())
        .insert_header((
            "Content-Disposition",
            format!("inline; filename=\"{}\"", meta.original_name),
        ))
        .body(data))
}
