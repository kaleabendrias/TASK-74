use diesel::PgConnection;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::model::*;
use crate::repository::{export as export_repo, import_jobs as repo};

/// Creates a new queued import job for the given file path and job type.
pub fn create_import_job(
    conn: &mut PgConnection,
    file_path: &str,
    job_type: &str,
    user_id: Uuid,
) -> Result<ImportJobResponse, ApiError> {
    let new = repo::NewImportJob {
        job_type,
        file_path,
        status: "queued",
        created_by: user_id,
    };

    let row = repo::insert_job(conn, &new)?;
    Ok(job_to_response(&row))
}

/// Retrieves an import job's current status and progress by ID.
pub fn get_import_job(
    conn: &mut PgConnection,
    id: Uuid,
) -> Result<ImportJobResponse, ApiError> {
    let row = repo::find_job(conn, id)?;
    Ok(job_to_response(&row))
}

/// Creates a pending export approval request for the specified export type.
pub fn create_export_request(
    conn: &mut PgConnection,
    export_type: &str,
    user_id: Uuid,
) -> Result<ExportApprovalResponse, ApiError> {
    let new = export_repo::NewExportApproval {
        export_type,
        requested_by: user_id,
        status: "pending",
    };

    let row = export_repo::insert_approval(conn, &new)?;
    Ok(approval_to_response(&row))
}

/// Approves a pending export request, enforcing that the approver differs from the requester.
pub fn approve_export(
    conn: &mut PgConnection,
    id: Uuid,
    approver_id: Uuid,
    approver_username: &str,
) -> Result<ExportApprovalResponse, ApiError> {
    let existing = export_repo::find_approval(conn, id)?;
    if existing.status != "pending" {
        return Err(ApiError::unprocessable(
            "INVALID_STATUS",
            "Only pending exports can be approved",
        ));
    }
    if existing.requested_by == approver_id {
        return Err(ApiError::forbidden(
            "Cannot approve your own export request",
        ));
    }

    let watermark = format!("{}@{}", approver_username, chrono::Utc::now().format("%Y%m%d%H%M%S"));
    let row = export_repo::approve_export(conn, id, approver_id, &watermark)?;
    Ok(approval_to_response(&row))
}

/// Retrieves an export approval record by ID.
pub fn get_export_approval(
    conn: &mut PgConnection,
    id: Uuid,
) -> Result<export_repo::ExportApprovalRow, ApiError> {
    export_repo::find_approval(conn, id).map_err(|_| ApiError::not_found("Export approval"))
}

// ── Helpers ──

fn job_to_response(row: &repo::ImportJobRow) -> ImportJobResponse {
    ImportJobResponse {
        id: row.id,
        job_type: row.job_type.clone(),
        status: row.status.clone(),
        total_rows: row.total_rows,
        processed_rows: row.processed_rows,
        progress_percent: row.progress_percent,
        retries: row.retries,
        failure_log: row.failure_log.clone(),
        committed: row.committed,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn approval_to_response(row: &export_repo::ExportApprovalRow) -> ExportApprovalResponse {
    ExportApprovalResponse {
        id: row.id,
        export_type: row.export_type.clone(),
        requested_by: row.requested_by,
        approved_by: row.approved_by,
        watermark_text: row.watermark_text.clone(),
        status: row.status.clone(),
        created_at: row.created_at,
    }
}
