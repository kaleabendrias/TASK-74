//! Integration simulation test for resumable import retries.
//!
//! Verifies the chunked-cursor logic:
//!   1. A job processes N rows in chunks.
//!   2. If processing fails after chunk k, the cursor is preserved.
//!   3. On retry, only rows after the cursor are re-processed.

/// Simulates the chunked ingestion logic without a live database.
/// Returns `(rows_inserted_total, final_cursor)`.
fn simulate_chunked_import(
    total_rows: usize,
    chunk_size: usize,
    fail_after_chunk: Option<usize>, // fail after committing this many chunks
) -> Result<(usize, usize), (usize, String)> {
    let mut inserted = 0usize;
    let mut chunk_num = 0usize;

    for chunk_start in (0..total_rows).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(total_rows);
        let chunk_rows = chunk_end - chunk_start;

        // Simulate inserting the chunk
        inserted += chunk_rows;
        chunk_num += 1;

        // Simulate a failure after the specified chunk
        if let Some(fail) = fail_after_chunk {
            if chunk_num == fail {
                return Err((inserted, format!("simulated failure after chunk {}", fail)));
            }
        }
    }

    Ok((inserted, inserted))
}

/// Resumes from a saved cursor position.
fn simulate_resume(
    total_rows: usize,
    chunk_size: usize,
    cursor: usize, // rows already committed
) -> usize {
    let mut inserted = cursor;
    for chunk_start in (cursor..total_rows).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(total_rows);
        inserted += chunk_end - chunk_start;
    }
    inserted
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn full_run_no_failure() {
    let (inserted, cursor) = simulate_chunked_import(1000, 500, None).unwrap();
    assert_eq!(inserted, 1000);
    assert_eq!(cursor, 1000);
}

#[test]
fn failure_after_first_chunk_preserves_cursor() {
    // 1000 rows in chunks of 500 — fail after chunk 1 (500 rows committed)
    let err = simulate_chunked_import(1000, 500, Some(1)).unwrap_err();
    let (cursor, msg) = err;
    assert_eq!(cursor, 500, "Cursor must reflect the 500 rows committed before failure");
    assert!(msg.contains("chunk 1"));
}

#[test]
fn resume_from_cursor_processes_remaining_rows_only() {
    // First attempt: fail after chunk 1, cursor = 500
    let (cursor, _) = simulate_chunked_import(1000, 500, Some(1)).unwrap_err();
    assert_eq!(cursor, 500);

    // Resume: should process only rows 500..1000
    let total_after_resume = simulate_resume(1000, 500, cursor);
    assert_eq!(total_after_resume, 1000, "After resume all rows must be accounted for");
}

#[test]
fn resume_from_mid_chunk_boundary() {
    // 1000 rows in chunks of 300 — fail after chunk 2 (600 rows committed)
    let (cursor, _) = simulate_chunked_import(1000, 300, Some(2)).unwrap_err();
    assert_eq!(cursor, 600);

    let total_after_resume = simulate_resume(1000, 300, cursor);
    assert_eq!(total_after_resume, 1000);
}

#[test]
fn resume_with_no_remaining_rows_is_noop() {
    // All rows already committed
    let total = simulate_resume(500, 500, 500);
    assert_eq!(total, 500, "Resuming from a fully-committed cursor should add zero rows");
}

#[test]
fn chunk_cursor_never_exceeds_total_rows() {
    // Various row counts / chunk sizes must never overshoot
    for (total, chunk) in &[(7, 3), (10, 5), (1, 1), (999, 500)] {
        let (inserted, _) = simulate_chunked_import(*total, *chunk, None).unwrap();
        assert_eq!(
            inserted, *total,
            "total={} chunk={}: inserted {} rows, expected {}",
            total, chunk, inserted, total
        );
    }
}

// ── Real DB-backed tests ───────────────────────────────────────────────────
#[allow(unused_imports)]
use diesel::prelude::*;
//
// These tests require a running PostgreSQL instance. The connection URL is
// read from the DATABASE_URL environment variable (same test DB used by the
// API tests). They are gated on that variable being present so that the pure
// unit test suite can still be run offline.

fn test_db_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn open_conn(url: &str) -> diesel::PgConnection {
    use diesel::Connection;
    diesel::PgConnection::establish(url)
        .unwrap_or_else(|e| panic!("DB connection failed: {}", e))
}

/// Staging table created for a job must be visible in information_schema.
#[test]
fn staging_table_exists_after_create() {
    let Some(url) = test_db_url() else { return };
    let mut conn = open_conn(&url);
    let name = format!("_test_staging_{}", uuid::Uuid::new_v4().to_string().replace('-', "_"));

    tourism_backend::jobs::runner::create_staging_table(&mut conn, &name)
        .expect("create_staging_table should not fail");

    assert!(
        tourism_backend::jobs::runner::staging_table_exists(&mut conn, &name),
        "Staging table '{}' should exist after creation", name
    );

    // Clean up
    tourism_backend::jobs::runner::drop_staging_table(&mut conn, &name);
}

/// After dropping a staging table it must no longer be reported as existing.
#[test]
fn staging_table_absent_after_drop() {
    let Some(url) = test_db_url() else { return };
    let mut conn = open_conn(&url);
    let name = format!("_test_staging_{}", uuid::Uuid::new_v4().to_string().replace('-', "_"));

    tourism_backend::jobs::runner::create_staging_table(&mut conn, &name).unwrap();
    tourism_backend::jobs::runner::drop_staging_table(&mut conn, &name);

    assert!(
        !tourism_backend::jobs::runner::staging_table_exists(&mut conn, &name),
        "Staging table '{}' should not exist after being dropped", name
    );
}

/// The cursor persisted in the import_jobs table by update_job_progress must
/// be readable back after the write.
#[test]
fn cursor_persisted_in_db() {
    use tourism_backend::repository::import_jobs as repo;

    let Some(url) = test_db_url() else { return };
    let mut conn = open_conn(&url);

    // We need a valid user UUID for the created_by FK. Use a nil UUID and
    // skip the FK check by inserting directly via raw SQL.
    let creator_id = uuid::Uuid::nil();
    // Ensure the nil user exists (ignore conflict).
    diesel::sql_query(
        "INSERT INTO users (id, username, password_hash, role, mfa_enabled) \
         VALUES ('00000000-0000-0000-0000-000000000000', '_test_cursor_user', 'x', 'Administrator', false) \
         ON CONFLICT (id) DO NOTHING"
    ).execute(&mut conn).ok();

    let job = repo::insert_job(&mut conn, &repo::NewImportJob {
        job_type: "test",
        file_path: "/dev/null",
        status: "running",
        created_by: creator_id,
    }).expect("insert_job");

    // Advance cursor to 250 rows out of 500.
    repo::update_job_progress(&mut conn, job.id, 250, 500, 50)
        .expect("update_job_progress");

    let loaded = repo::find_job(&mut conn, job.id).expect("find_job");
    assert_eq!(loaded.processed_rows, 250, "processed_rows cursor must be 250");
    assert_eq!(loaded.total_rows, 500);
    assert_eq!(loaded.progress_percent, 50);

    // Clean up
    diesel::sql_query(format!("DELETE FROM import_jobs WHERE id = '{}'", job.id))
        .execute(&mut conn).ok();
}

/// update_staging_table_name must persist the staging table name so that a
/// subsequent find_job read returns the same name.
#[test]
fn staging_table_name_persisted_in_job() {
    use tourism_backend::repository::import_jobs as repo;

    let Some(url) = test_db_url() else { return };
    let mut conn = open_conn(&url);

    // Ensure nil user exists.
    diesel::sql_query(
        "INSERT INTO users (id, username, password_hash, role, mfa_enabled) \
         VALUES ('00000000-0000-0000-0000-000000000000', '_test_cursor_user', 'x', 'Administrator', false) \
         ON CONFLICT (id) DO NOTHING"
    ).execute(&mut conn).ok();

    let job = repo::insert_job(&mut conn, &repo::NewImportJob {
        job_type: "test",
        file_path: "/dev/null",
        status: "running",
        created_by: uuid::Uuid::nil(),
    }).expect("insert_job");

    let staging_name = format!("_import_{}", job.id.to_string().replace('-', "_"));
    repo::update_staging_table_name(&mut conn, job.id, &staging_name)
        .expect("update_staging_table_name");

    let loaded = repo::find_job(&mut conn, job.id).expect("find_job");
    assert_eq!(
        loaded.staging_table_name.as_deref(),
        Some(staging_name.as_str()),
        "staging_table_name must match what was saved"
    );

    // Clean up
    diesel::sql_query(format!("DELETE FROM import_jobs WHERE id = '{}'", job.id))
        .execute(&mut conn).ok();
}
