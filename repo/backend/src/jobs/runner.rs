use calamine::{open_workbook, Reader, Xlsx};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::time::Duration;
use tokio::time;

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

fn poll_and_process(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;
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

                // Re-queue if retries remain
                if job.retries + 1 < job.max_retries {
                    import_jobs::requeue_failed_job(&mut conn, job.id)?;
                    tracing::info!(job_id = %job.id, "Re-queued failed job for retry");
                }
            }
        }
    }

    Ok(())
}

fn process_xlsx_job(
    conn: &mut PgConnection,
    job: &import_jobs::ImportJobRow,
) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::Connection;

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
    import_jobs::update_job_progress(conn, job.id, 0, total_rows, 0)?;

    // Parse all rows, collecting errors
    let mut valid_rows = Vec::new();
    let mut errors = Vec::new();

    for (i, row) in rows.iter().skip(1).enumerate() {
        let mut obj = serde_json::Map::new();
        let mut row_valid = true;

        for (j, cell) in row.iter().enumerate() {
            let key = header_names.get(j).cloned().unwrap_or_else(|| format!("col_{}", j));
            let val = cell.to_string();
            if key == "item_name" && val.trim().is_empty() {
                errors.push(format!("Row {}: missing required field '{}'", i + 1, key));
                row_valid = false;
            }
            obj.insert(key, serde_json::Value::String(val));
        }

        if row_valid {
            valid_rows.push((i + 1, serde_json::Value::Object(obj)));
        }
    }

    if !errors.is_empty() && valid_rows.is_empty() {
        let log = errors.join("\n");
        return Err(format!("All rows invalid:\n{}", log).into());
    }

    // Atomic commit: transaction wraps staging + target insert
    let result = conn.transaction(|tx_conn| {
        let staging_table = format!("staging_{}", job.id.to_string().replace('-', "_"));

        diesel::sql_query(format!(
            "CREATE TEMP TABLE {} (row_num INT, data JSONB)", staging_table
        )).execute(tx_conn)?;

        for (row_num, data) in &valid_rows {
            diesel::sql_query(format!(
                "INSERT INTO {} (row_num, data) VALUES ({}, $1)", staging_table, row_num
            ))
            .bind::<diesel::sql_types::Jsonb, _>(data.clone())
            .execute(tx_conn)?;
        }

        // Commit staged data into audit_log as a record of the import
        diesel::sql_query(format!(
            "INSERT INTO audit_log (id, actor_id, action, entity_type, entity_id, detail, created_at) \
             SELECT gen_random_uuid(), '{}', 'import', '{}', gen_random_uuid(), data, now() \
             FROM {}",
            job.created_by, job.job_type, staging_table
        )).execute(tx_conn)?;

        diesel::sql_query(format!("DROP TABLE IF EXISTS {}", staging_table))
            .execute(tx_conn)?;

        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(()) => {
            let percent = if errors.is_empty() { 100i16 } else {
                ((valid_rows.len() as f64 / total_rows as f64) * 100.0) as i16
            };
            import_jobs::update_job_progress(conn, job.id, valid_rows.len() as i32, total_rows, percent)?;

            if !errors.is_empty() {
                let log = errors.join("\n");
                import_jobs::mark_job_failed(conn, job.id, &log)?;
                return Err(format!("Partial failure: {} rows failed:\n{}", errors.len(), log).into());
            }
        }
        Err(e) => {
            tracing::error!(job_id = %job.id, error = %e, "Transaction rolled back");
            return Err(format!("Transaction failed: {}", e).into());
        }
    }

    Ok(())
}
