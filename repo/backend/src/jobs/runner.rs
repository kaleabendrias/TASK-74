use calamine::{open_workbook, Reader, Xlsx};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::time::Duration;
use tokio::time;
use uuid;

use crate::repository::import_jobs;

type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Spawns a background Tokio task that polls for queued import jobs and processes them.
pub fn spawn_job_runner(pool: DbPool) {
    tokio::spawn(async move {
        let poll_interval = Duration::from_secs(10);
        loop {
            if let Err(e) = poll_and_process(&pool) {
                tracing::error!(error = %e, "Job runner cycle failed");
            }
            time::sleep(poll_interval).await;
        }
    });
}

/// Jobs stuck in "running" for longer than this threshold are assumed orphaned
/// by a process crash and will be reset to "queued" for re-processing.
const STALE_JOB_TIMEOUT_SECS: i64 = 600; // 10 minutes

fn poll_and_process(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    // ── Crash recovery: reset stale running jobs ──────────────────────────
    // If a previous process crashed mid-job the status stays "running"
    // indefinitely. Resetting these to "queued" re-enters them into the
    // normal retry flow without needing manual intervention.
    match import_jobs::reset_stale_running_jobs(&mut conn, STALE_JOB_TIMEOUT_SECS) {
        Ok(n) if n > 0 => tracing::warn!(
            count = n,
            timeout_secs = STALE_JOB_TIMEOUT_SECS,
            "Reset {} stale running import job(s) — likely orphaned by a process crash",
            n
        ),
        Ok(_) => {}
        Err(e) => tracing::error!(error = %e, "Failed to reset stale running jobs"),
    }

    let jobs = import_jobs::find_queued_jobs(&mut conn, 5)?;

    for job in jobs {
        tracing::info!(job_id = %job.id, "Processing import job");
        import_jobs::update_job_status(&mut conn, job.id, "running")?;

        match process_xlsx_job(&mut conn, &job) {
            Ok(()) => {
                import_jobs::mark_job_completed(&mut conn, job.id, true)?;
                tracing::info!(job_id = %job.id, "Import job completed");
            }
            Err(e) => {
                let msg = format!("{}", e);
                tracing::error!(job_id = %job.id, error = %msg, "Import job failed");
                import_jobs::mark_job_failed(&mut conn, job.id, &msg)?;

                // Re-queue if under max retries (the SQL filter enforces the limit)
                let requeued = import_jobs::requeue_failed_job(&mut conn, job.id)?;
                if requeued > 0 {
                    tracing::info!(job_id = %job.id, "Re-queued failed job for retry");
                }
            }
        }
    }

    Ok(())
}

/// Chunk size for chunked staging inserts. After each chunk the cursor
/// (processed_rows) is written to the DB, enabling resume on retry.
const IMPORT_CHUNK_SIZE: usize = 500;

fn process_xlsx_job(
    conn: &mut PgConnection,
    job: &import_jobs::ImportJobRow,
) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::Connection;

    // ── Determine whether this is a fresh start or a resume ──
    //
    // If `staging_table_name` is set the job failed mid-processing on a
    // previous attempt. We check whether the persistent staging table still
    // exists; if it does we skip validation and resume row ingestion from
    // `processed_rows` (the committed cursor).  If the table was lost (e.g. a
    // DB restart dropped it) we fall back to a full re-run.

    let staging = match &job.staging_table_name {
        Some(name) => {
            // Verify the table still exists in the public schema
            let exists: bool = diesel::sql_query(format!(
                "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
                 WHERE table_schema = 'public' AND table_name = '{}')", name
            ))
            .load::<BoolRow>(conn)
            .map(|rows| rows.first().map(|r| r.exists).unwrap_or(false))
            .unwrap_or(false);

            if exists {
                tracing::info!(
                    job_id = %job.id,
                    staging = %name,
                    cursor = job.processed_rows,
                    "Resuming import job from saved cursor"
                );
                name.clone()
            } else {
                tracing::warn!(
                    job_id = %job.id,
                    staging = %name,
                    "Staging table no longer exists — restarting from row 0"
                );
                String::new()
            }
        }
        None => String::new(),
    };

    let is_resume = !staging.is_empty();
    let resume_cursor = if is_resume { job.processed_rows as usize } else { 0 };

    // ── Parse workbook ──
    let mut workbook: Xlsx<_> = open_workbook(&job.file_path)?;
    let sheet_name = workbook.sheet_names().first().cloned()
        .ok_or("No sheets in workbook")?;
    let range = workbook.worksheet_range(&sheet_name)?;

    let rows: Vec<_> = range.rows().collect();
    if rows.len() <= 1 {
        return Err("No data rows in spreadsheet".into());
    }

    let header_names: Vec<String> = rows[0].iter()
        .map(|c| c.to_string().trim().to_lowercase())
        .collect();
    let total_rows = (rows.len() - 1) as i32;
    if total_rows > 10_000 {
        return Err(format!("Import exceeds maximum of 10,000 rows ({} rows found)", total_rows).into());
    }

    // ── Phase 1: Validate ALL rows (skipped when resuming with a live table) ──
    let parsed_rows: Vec<serde_json::Map<String, serde_json::Value>> = if !is_resume {
        import_jobs::update_job_progress(conn, job.id, 0, total_rows, 0)?;

        let mut pr = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for (i, row) in rows.iter().skip(1).enumerate() {
            let row_num = i + 1;
            let mut obj = serde_json::Map::new();

            for (j, cell) in row.iter().enumerate() {
                let key = header_names.get(j).cloned().unwrap_or_else(|| format!("col_{}", j));
                let val = cell.to_string().trim().to_string();
                obj.insert(key, serde_json::Value::String(val));
            }

            let row_errors = validate_import_row_fields(&obj);
            if !row_errors.is_empty() {
                errors.push(format!("Row {}: {}", row_num, row_errors.join("; ")));
            }
            pr.push(obj);
        }

        // Fail fast: if ANY row has errors, abort before touching the target table
        if !errors.is_empty() {
            let log = errors.join("\n");
            import_jobs::mark_job_failed(conn, job.id, &log)?;
            return Err(format!("{} of {} rows failed validation:\n{}", errors.len(), total_rows, log).into());
        }

        pr
    } else {
        // Resume path: re-parse rows but skip already-committed rows below
        rows.iter().skip(1).map(|row| {
            let mut obj = serde_json::Map::new();
            for (j, cell) in row.iter().enumerate() {
                let key = header_names.get(j).cloned().unwrap_or_else(|| format!("col_{}", j));
                let val = cell.to_string().trim().to_string();
                obj.insert(key, serde_json::Value::String(val));
            }
            obj
        }).collect()
    };

    // ── Phase 2: Create persistent staging table (new jobs only) ──
    let staging = if !is_resume {
        let name = format!("_import_{}", job.id.to_string().replace('-', "_"));
        diesel::sql_query(format!(
            "CREATE TABLE IF NOT EXISTS {} (\
             facility_id UUID NOT NULL, warehouse_id UUID NOT NULL, bin_id UUID NOT NULL, \
             item_name TEXT NOT NULL, lot_number TEXT NOT NULL, quantity_on_hand INT NOT NULL\
             )", name
        )).execute(conn)?;
        // Persist the staging table name so retries can resume
        import_jobs::update_staging_table_name(conn, job.id, &name)?;
        name
    } else {
        staging
    };

    // ── Phase 3: Chunked row ingestion into staging (resumable) ──
    let rows_to_insert = &parsed_rows[resume_cursor..];
    let mut committed_so_far = resume_cursor as i32;

    for chunk in rows_to_insert.chunks(IMPORT_CHUNK_SIZE) {
        for obj in chunk {
            let item = obj.get("item_name").and_then(|v| v.as_str()).unwrap_or("");
            let lot  = obj.get("lot_number").and_then(|v| v.as_str()).unwrap_or("IMPORTED");
            let qty: i32 = obj.get("quantity_on_hand").and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()).unwrap_or(0);
            let fid = obj.get("facility_id").and_then(|v| v.as_str()).unwrap_or("");
            let wid = obj.get("warehouse_id").and_then(|v| v.as_str()).unwrap_or("");
            let bid = obj.get("bin_id").and_then(|v| v.as_str()).unwrap_or("");

            diesel::sql_query(format!(
                "INSERT INTO {} (facility_id,warehouse_id,bin_id,item_name,lot_number,quantity_on_hand) \
                 VALUES ($1::uuid,$2::uuid,$3::uuid,$4,$5,$6)", staging
            ))
            .bind::<diesel::sql_types::Text, _>(fid)
            .bind::<diesel::sql_types::Text, _>(wid)
            .bind::<diesel::sql_types::Text, _>(bid)
            .bind::<diesel::sql_types::Text, _>(item)
            .bind::<diesel::sql_types::Text, _>(lot)
            .bind::<diesel::sql_types::Integer, _>(qty)
            .execute(conn)?;
        }

        // Commit the cursor after each chunk so retries can resume here
        committed_so_far += chunk.len() as i32;
        let pct = ((committed_so_far as f64 / total_rows as f64) * 90.0) as i16; // reserve last 10% for final commit
        import_jobs::update_job_progress(conn, job.id, committed_so_far, total_rows, pct)?;
    }

    // ── Phase 4: Atomic final commit — staging → target ──
    conn.transaction(|tx| {
        diesel::sql_query(format!(
            "INSERT INTO inventory_lots \
             (id, facility_id, warehouse_id, bin_id, item_name, lot_number, \
              quantity_on_hand, quantity_reserved, created_at, updated_at) \
             SELECT gen_random_uuid(), facility_id, warehouse_id, bin_id, \
                    item_name, lot_number, quantity_on_hand, 0, now(), now() \
             FROM {}", staging
        )).execute(tx)?;

        // Audit entry
        diesel::sql_query(format!(
            "INSERT INTO audit_log (id, actor_id, action, entity_type, detail, created_at) \
             VALUES (gen_random_uuid(), '{}', 'bulk_import', 'inventory_lots', \
             '{{}}'::jsonb, now())", job.created_by
        )).execute(tx)?;

        Ok::<_, diesel::result::Error>(())
    }).map_err(|e| {
        tracing::error!(job_id = %job.id, error = %e, "Import final commit rolled back");
        format!("Final commit rolled back: {}", e)
    })?;

    // Drop the persistent staging table now that data is committed
    diesel::sql_query(format!("DROP TABLE IF EXISTS {}", staging)).execute(conn)?;

    import_jobs::update_job_progress(conn, job.id, total_rows, total_rows, 100)?;
    Ok(())
}

/// Returns `true` if a staging table with the given name exists in the public
/// schema. Exposed as `pub` so that integration tests can assert on it.
pub fn staging_table_exists(conn: &mut PgConnection, name: &str) -> bool {
    diesel::sql_query(format!(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name = '{}')", name
    ))
    .load::<BoolRow>(conn)
    .map(|rows| rows.first().map(|r| r.exists).unwrap_or(false))
    .unwrap_or(false)
}

/// Creates an empty import staging table with the standard schema.
/// Exposed as `pub` so that integration tests can set up fixtures directly.
pub fn create_staging_table(conn: &mut PgConnection, name: &str) -> Result<(), diesel::result::Error> {
    diesel::sql_query(format!(
        "CREATE TABLE IF NOT EXISTS {} (\
         facility_id UUID NOT NULL, warehouse_id UUID NOT NULL, bin_id UUID NOT NULL, \
         item_name TEXT NOT NULL, lot_number TEXT NOT NULL, quantity_on_hand INT NOT NULL\
         )", name
    )).execute(conn).map(|_| ())
}

/// Drops a staging table. Exposed as `pub` for test cleanup.
pub fn drop_staging_table(conn: &mut PgConnection, name: &str) {
    diesel::sql_query(format!("DROP TABLE IF EXISTS {}", name))
        .execute(conn)
        .ok();
}

/// Helper for reading a boolean from a raw SQL EXISTS query.
#[derive(diesel::QueryableByName, Debug)]
struct BoolRow {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    exists: bool,
}

/// Validates a single import row map, returning a list of human-readable error
/// strings. An empty return value means the row is valid.
///
/// Rules enforced:
/// - `item_name`: required, non-empty.
/// - `quantity_on_hand`: if present, must parse as `i32`.
/// - `facility_id`, `warehouse_id`, `bin_id`: required non-empty; if non-empty, must
///   be a valid UUID.
pub fn validate_import_row_fields(obj: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    let mut errors = Vec::new();

    // item_name is required and non-empty
    match obj.get("item_name").and_then(|v| v.as_str()) {
        None | Some("") => errors.push("missing required field 'item_name'".to_string()),
        _ => {}
    }

    // quantity_on_hand must parse as i32 when present
    if let Some(val) = obj.get("quantity_on_hand").and_then(|v| v.as_str()) {
        if val.parse::<i32>().is_err() {
            errors.push(format!("invalid integer for 'quantity_on_hand': '{}'", val));
        }
    }

    // UUID fields: required non-empty and must be valid UUIDs
    for key in &["facility_id", "warehouse_id", "bin_id"] {
        match obj.get(*key).and_then(|v| v.as_str()) {
            None | Some("") => errors.push(format!("missing required field '{}'", key)),
            Some(val) if uuid::Uuid::parse_str(val).is_err() => {
                errors.push(format!("invalid UUID for '{}': '{}'", key, val));
            }
            _ => {}
        }
    }

    errors
}

/// Spawns a background task that publishes resources whose scheduled_publish_at has arrived.
pub fn spawn_scheduled_publisher(pool: DbPool) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(30);
        loop {
            if let Err(e) = publish_scheduled(&pool) {
                tracing::error!(error = %e, "Scheduled publisher cycle failed");
            }
            time::sleep(interval).await;
        }
    });
}

fn publish_scheduled(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;
    let now = chrono::Utc::now();

    let count = diesel::sql_query(
        "UPDATE resources r SET state = 'published', updated_at = now() \
         WHERE r.scheduled_publish_at <= $1 \
         AND r.scheduled_publish_at IS NOT NULL \
         AND r.state = 'in_review' \
         AND EXISTS ( \
             SELECT 1 FROM review_decisions rd \
             WHERE rd.entity_type = 'resource' \
             AND rd.entity_id = r.id \
             AND rd.decision = 'approved' \
         )"
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)?;

    if count > 0 {
        tracing::info!(count = count, "Published {} approved scheduled resources", count);
    }
    Ok(())
}
