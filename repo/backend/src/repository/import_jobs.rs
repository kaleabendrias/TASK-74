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

pub fn insert_job(conn: &mut PgConnection, new: &NewImportJob) -> QueryResult<ImportJobRow> {
    diesel::insert_into(import_jobs::table)
        .values(new)
        .returning(ImportJobRow::as_returning())
        .get_result(conn)
}

pub fn find_job(conn: &mut PgConnection, id: Uuid) -> QueryResult<ImportJobRow> {
    import_jobs::table
        .find(id)
        .select(ImportJobRow::as_select())
        .first(conn)
}

pub fn find_queued_jobs(conn: &mut PgConnection, limit: i64) -> QueryResult<Vec<ImportJobRow>> {
    import_jobs::table
        .filter(import_jobs::status.eq("queued"))
        .order(import_jobs::created_at.asc())
        .limit(limit)
        .select(ImportJobRow::as_select())
        .load(conn)
}

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

pub fn count_queued(conn: &mut PgConnection) -> QueryResult<i64> {
    import_jobs::table
        .filter(import_jobs::status.eq_any(vec!["queued", "running"]))
        .count()
        .get_result(conn)
}
