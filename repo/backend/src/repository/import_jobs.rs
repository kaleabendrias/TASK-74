use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::import_jobs;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = import_jobs)]
pub struct ImportJobRow {
    pub id: Uuid,
    pub job_type: String,
    pub file_path: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub progress_percent: i16,
    pub status: String,
    pub retries: i32,
    pub max_retries: i32,
    pub failure_log: Option<String>,
    pub staging_table_name: Option<String>,
    pub committed: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = import_jobs)]
pub struct NewImportJob<'a> {
    pub job_type: &'a str,
    pub file_path: &'a str,
    pub status: &'a str,
    pub created_by: Uuid,
}

/// Inserts a new import job into the database.
pub fn insert_job(conn: &mut PgConnection, new: &NewImportJob) -> QueryResult<ImportJobRow> {
    diesel::insert_into(import_jobs::table)
        .values(new)
        .returning(ImportJobRow::as_returning())
        .get_result(conn)
}

/// Finds an import job by its unique ID.
pub fn find_job(conn: &mut PgConnection, id: Uuid) -> QueryResult<ImportJobRow> {
    import_jobs::table
        .find(id)
        .select(ImportJobRow::as_select())
        .first(conn)
}

/// Retrieves up to `limit` queued import jobs ordered by creation time.
pub fn find_queued_jobs(conn: &mut PgConnection, limit: i64) -> QueryResult<Vec<ImportJobRow>> {
    import_jobs::table
        .filter(import_jobs::status.eq("queued"))
        .order(import_jobs::created_at.asc())
        .limit(limit)
        .select(ImportJobRow::as_select())
        .load(conn)
}

/// Updates the status of an import job.
pub fn update_job_status(
    conn: &mut PgConnection,
    id: Uuid,
    status: &str,
) -> QueryResult<usize> {
    diesel::update(import_jobs::table.find(id))
        .set((
            import_jobs::status.eq(status),
            import_jobs::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
}

/// Updates the progress counters and percentage of an import job.
pub fn update_job_progress(
    conn: &mut PgConnection,
    id: Uuid,
    processed: i32,
    total: i32,
    percent: i16,
) -> QueryResult<usize> {
    diesel::update(import_jobs::table.find(id))
        .set((
            import_jobs::processed_rows.eq(processed),
            import_jobs::total_rows.eq(total),
            import_jobs::progress_percent.eq(percent),
            import_jobs::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
}

/// Marks an import job as completed and sets its committed flag.
pub fn mark_job_completed(conn: &mut PgConnection, id: Uuid, committed: bool) -> QueryResult<usize> {
    diesel::update(import_jobs::table.find(id))
        .set((
            import_jobs::status.eq("completed"),
            import_jobs::committed.eq(committed),
            import_jobs::progress_percent.eq(100i16),
            import_jobs::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
}

/// Marks an import job as failed and records the failure reason.
pub fn mark_job_failed(
    conn: &mut PgConnection,
    id: Uuid,
    failure: &str,
) -> QueryResult<usize> {
    diesel::update(import_jobs::table.find(id))
        .set((
            import_jobs::status.eq("failed"),
            import_jobs::failure_log.eq(Some(failure)),
            import_jobs::retries.eq(import_jobs::retries + 1),
            import_jobs::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
}

/// Re-queues a failed job if it has not exceeded its maximum retry count.
pub fn requeue_failed_job(conn: &mut PgConnection, id: Uuid) -> QueryResult<usize> {
    diesel::update(
        import_jobs::table
            .find(id)
            .filter(import_jobs::retries.lt(import_jobs::max_retries)),
    )
    .set((
        import_jobs::status.eq("queued"),
        import_jobs::updated_at.eq(Utc::now()),
    ))
    .execute(conn)
}

/// Counts the number of queued or currently running import jobs.
pub fn count_queued(conn: &mut PgConnection) -> QueryResult<i64> {
    import_jobs::table
        .filter(import_jobs::status.eq_any(vec!["queued", "running"]))
        .count()
        .get_result(conn)
}
