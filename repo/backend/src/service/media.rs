use diesel::PgConnection;
use std::path::Path;
use uuid::Uuid;

use crate::config::UploadConfig;
use crate::crypto::sha256;
use crate::errors::ApiError;
use crate::model::MediaFileResponse;
use crate::repository::media as repo;

/// Validates, stores, and records an uploaded media file (extension, MIME, size, checksum).
pub fn process_upload(
    conn: &mut PgConnection,
    config: &UploadConfig,
    original_name: &str,
    data: &[u8],
    user_id: Uuid,
) -> Result<MediaFileResponse, ApiError> {
    let ext = Path::new(original_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    // Derive allowed extensions from config MIME types
    let allowed_extensions: Vec<&str> = config.allowed_mimes.iter().flat_map(|mime| {
        match mime.as_str() {
            "image/jpeg" => vec!["jpg", "jpeg"],
            "image/png" => vec!["png"],
            "image/webp" => vec!["webp"],
            "video/mp4" => vec!["mp4"],
            "application/pdf" => vec!["pdf"],
            _ => vec![],
        }
    }).collect();

    if !allowed_extensions.contains(&ext.as_str()) {
        return Err(ApiError::unprocessable(
            "INVALID_FILE_TYPE",
            &format!("File extension '{}' not allowed. Allowed: {}", ext, allowed_extensions.join(", ")),
        ));
    }

    // MIME type sniffing
    let inferred = infer::get(data);
    let mime_type = inferred
        .map(|t| t.mime_type().to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Validate MIME matches extension
    let mime_ok = match ext.as_str() {
        "jpg" | "jpeg" => mime_type == "image/jpeg",
        "png" => mime_type == "image/png",
        "webp" => mime_type == "image/webp",
        "mp4" => mime_type == "video/mp4",
        "pdf" => mime_type == "application/pdf",
        _ => false,
    };
    if !mime_ok {
        return Err(ApiError::unprocessable(
            "MIME_MISMATCH",
            &format!(
                "File extension '{}' does not match detected MIME type '{}'",
                ext, mime_type
            ),
        ));
    }

    // Size check
    if data.len() > config.max_size_bytes {
        return Err(ApiError::unprocessable(
            "FILE_TOO_LARGE",
            &format!("File size {} bytes exceeds maximum of {} bytes", data.len(), config.max_size_bytes),
        ));
    }

    // Compute checksum
    let checksum = sha256::hash_bytes(data);

    // Sanitize and store
    let safe_name = sanitize_filename::sanitize(original_name);
    let file_id = Uuid::new_v4();
    let stored_name = format!("{}_{}", file_id, safe_name);
    let stored_path = format!("{}/{}", config.storage_path, stored_name);

    // Write to disk
    std::fs::create_dir_all(&config.storage_path).map_err(|e| {
        tracing::error!(error = %e, "Failed to create upload directory");
        ApiError::internal("Failed to store file")
    })?;
    std::fs::write(&stored_path, data).map_err(|e| {
        tracing::error!(error = %e, path = %stored_path, "Failed to write file");
        ApiError::internal("Failed to store file")
    })?;

    // DB record
    let new = repo::NewMediaFile {
        original_name: &safe_name,
        stored_path: &stored_path,
        mime_type: &mime_type,
        size_bytes: data.len() as i64,
        checksum_sha256: &checksum,
        uploaded_by: user_id,
    };

    let row = repo::insert(conn, &new)?;
    Ok(file_to_response(&row))
}

/// Retrieves a media file's metadata and raw bytes from disk by ID.
pub fn get_file(
    conn: &mut PgConnection,
    id: Uuid,
) -> Result<(repo::MediaFileRow, Vec<u8>), ApiError> {
    let row = repo::find_by_id(conn, id)?;
    let data = std::fs::read(&row.stored_path).map_err(|e| {
        tracing::error!(error = %e, path = %row.stored_path, "Failed to read file");
        ApiError::internal("File not found on disk")
    })?;
    Ok((row, data))
}

fn file_to_response(row: &repo::MediaFileRow) -> MediaFileResponse {
    MediaFileResponse {
        id: row.id,
        original_name: row.original_name.clone(),
        mime_type: row.mime_type.clone(),
        size_bytes: row.size_bytes,
        checksum_sha256: row.checksum_sha256.clone(),
        uploaded_by: row.uploaded_by,
        created_at: row.created_at,
    }
}
