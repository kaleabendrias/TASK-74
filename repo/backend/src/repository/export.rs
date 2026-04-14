use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::export_approvals;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = export_approvals)]
pub struct ExportApprovalRow {
    pub id: Uuid,
    pub export_type: String,
    pub requested_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub watermark_text: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = export_approvals)]
pub struct NewExportApproval<'a> {
    pub export_type: &'a str,
    pub requested_by: Uuid,
    pub status: &'a str,
}

/// Inserts a new export approval request into the database.
pub fn insert_approval(
    conn: &mut PgConnection,
    new: &NewExportApproval,
) -> QueryResult<ExportApprovalRow> {
    diesel::insert_into(export_approvals::table)
        .values(new)
        .returning(ExportApprovalRow::as_returning())
        .get_result(conn)
}

/// Finds an export approval record by its unique ID.
pub fn find_approval(conn: &mut PgConnection, id: Uuid) -> QueryResult<ExportApprovalRow> {
    export_approvals::table
        .find(id)
        .select(ExportApprovalRow::as_select())
        .first(conn)
}

/// Lists all export approvals with status = 'pending'.
pub fn list_pending(conn: &mut PgConnection) -> QueryResult<Vec<ExportApprovalRow>> {
    export_approvals::table
        .filter(export_approvals::status.eq("pending"))
        .order(export_approvals::created_at.desc())
        .select(ExportApprovalRow::as_select())
        .load(conn)
}

/// Approves an export request, recording the approver and watermark text.
pub fn approve_export(
    conn: &mut PgConnection,
    id: Uuid,
    approver: Uuid,
    watermark: &str,
) -> QueryResult<ExportApprovalRow> {
    diesel::update(export_approvals::table.find(id))
        .set((
            export_approvals::status.eq("approved"),
            export_approvals::approved_by.eq(Some(approver)),
            export_approvals::watermark_text.eq(Some(watermark)),
        ))
        .returning(ExportApprovalRow::as_returning())
        .get_result(conn)
}
