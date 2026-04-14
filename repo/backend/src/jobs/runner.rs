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

    // Parse all rows, collecting errors per-row
    let mut valid_rows: Vec<(i32, serde_json::Map<String, serde_json::Value>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (i, row) in rows.iter().skip(1).enumerate() {
        let row_num = (i + 1) as i32;
        let mut obj = serde_json::Map::new();
        let mut row_errors = Vec::new();

        for (j, cell) in row.iter().enumerate() {
            let key = header_names.get(j).cloned().unwrap_or_else(|| format!("col_{}", j));
            let val = cell.to_string().trim().to_string();
            obj.insert(key.clone(), serde_json::Value::String(val.clone()));

            // Validate required fields
            if key == "item_name" && val.is_empty() {
                row_errors.push(format!("missing required field 'item_name'"));
            }
            if key == "quantity_on_hand" {
                if val.parse::<i32>().is_err() {
                    row_errors.push(format!("invalid integer for 'quantity_on_hand': '{}'", val));
                }
            }
        }

        if row_errors.is_empty() {
            valid_rows.push((row_num, obj));
        } else {
            errors.push(format!("Row {}: {}", row_num, row_errors.join("; ")));
        }
    }

    if valid_rows.is_empty() {
        let log = errors.join("\n");
        return Err(format!("All {} rows invalid:\n{}", errors.len(), log).into());
    }

    // Atomic staging-to-target commit
    let insert_count = valid_rows.len() as i32;
    let result = conn.transaction(|tx_conn| {
        for (row_num, obj) in &valid_rows {
            // Extract fields for inventory_lots — use sensible defaults for missing columns
            let item_name = obj.get("item_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let lot_number = obj.get("lot_number").and_then(|v| v.as_str()).unwrap_or("IMPORTED").to_string();
            let qty: i32 = obj.get("quantity_on_hand").and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()).unwrap_or(0);
            let facility_id = obj.get("facility_id").and_then(|v| v.as_str()).unwrap_or("00000000-0000-0000-0000-000000000001");
            let warehouse_id = obj.get("warehouse_id").and_then(|v| v.as_str()).unwrap_or("00000000-0000-0000-0000-000000000002");
            let bin_id = obj.get("bin_id").and_then(|v| v.as_str()).unwrap_or("00000000-0000-0000-0000-000000000003");

            diesel::sql_query(
                "INSERT INTO inventory_lots (id, facility_id, warehouse_id, bin_id, item_name, lot_number, quantity_on_hand, quantity_reserved, created_at, updated_at) \
                 VALUES (gen_random_uuid(), $1::uuid, $2::uuid, $3::uuid, $4, $5, $6, 0, now(), now())"
            )
            .bind::<diesel::sql_types::Text, _>(facility_id)
            .bind::<diesel::sql_types::Text, _>(warehouse_id)
            .bind::<diesel::sql_types::Text, _>(bin_id)
            .bind::<diesel::sql_types::Text, _>(&item_name)
            .bind::<diesel::sql_types::Text, _>(&lot_number)
            .bind::<diesel::sql_types::Integer, _>(qty)
            .execute(tx_conn)?;
        }

        // Record the import in audit_log
        diesel::sql_query(format!(
            "INSERT INTO audit_log (id, actor_id, action, entity_type, detail, created_at) \
             VALUES (gen_random_uuid(), '{}', 'bulk_import', 'inventory_lots', \
             '{}'::jsonb, now())",
            job.created_by,
            serde_json::json!({"rows_imported": insert_count, "job_id": job.id.to_string()}).to_string().replace('\'', "''")
        )).execute(tx_conn)?;

        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(()) => {
            import_jobs::update_job_progress(conn, job.id, insert_count, total_rows,
                ((insert_count as f64 / total_rows as f64) * 100.0) as i16)?;
            if !errors.is_empty() {
                import_jobs::mark_job_failed(conn, job.id, &errors.join("\n"))?;
                return Err(format!("{} of {} rows failed validation", errors.len(), total_rows).into());
            }
        }
        Err(e) => {
            tracing::error!(job_id = %job.id, error = %e, "Import transaction rolled back");
            return Err(format!("Transaction rolled back: {}", e).into());
        }
    }

    Ok(())
}
