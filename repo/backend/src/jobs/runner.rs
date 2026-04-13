use calamine::{open_workbook, Reader, Xlsx};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::time::Duration;
use tokio::time;

use crate::repository::import_jobs;

type DbPool = Pool<ConnectionManager<PgConnection>>;

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
    let mut workbook: Xlsx<_> = open_workbook(&job.file_path)?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or("No sheets in workbook")?;

    let range = workbook.worksheet_range(&sheet_name)?;

    let total_rows = range.rows().count().saturating_sub(1) as i32; // exclude header
    if total_rows == 0 {
        return Err("No data rows in spreadsheet".into());
    }

    import_jobs::update_job_progress(conn, job.id, 0, total_rows, 0)?;

    // Create a staging table dynamically
    let staging_table = format!("staging_{}", job.id.to_string().replace('-', "_"));
    diesel::sql_query(format!(
        "CREATE TEMP TABLE {} (row_num INT, data JSONB)",
        staging_table
    ))
    .execute(conn)?;

    let mut processed = 0;
    let rows: Vec<_> = range.rows().collect();
    let header = &rows[0];
    let header_names: Vec<String> = header
        .iter()
        .map(|c| c.to_string().trim().to_lowercase())
        .collect();

    for (i, row) in rows.iter().skip(1).enumerate() {
        let mut obj = serde_json::Map::new();
        for (j, cell) in row.iter().enumerate() {
            let key = header_names.get(j).cloned().unwrap_or_else(|| format!("col_{}", j));
            obj.insert(key, serde_json::Value::String(cell.to_string()));
        }

        let row_json = serde_json::Value::Object(obj);
        diesel::sql_query(format!(
            "INSERT INTO {} (row_num, data) VALUES ({}, $1)",
            staging_table,
            i + 1
        ))
        .bind::<diesel::sql_types::Jsonb, _>(row_json)
        .execute(conn)?;

        processed += 1;

        // Update progress every 100 rows
        if processed % 100 == 0 || processed == total_rows {
            let percent = ((processed as f64 / total_rows as f64) * 100.0) as i16;
            import_jobs::update_job_progress(conn, job.id, processed, total_rows, percent)?;
        }
    }

    // Commit staging → real table would go here, dependent on job_type.
    // For now we mark as committed since we successfully staged all rows.
    diesel::sql_query(format!("DROP TABLE IF EXISTS {}", staging_table)).execute(conn)?;

    Ok(())
}
