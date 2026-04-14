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
    if total_rows > 10_000 {
        return Err(format!("Import exceeds maximum of 10,000 rows ({} rows found)", total_rows).into());
    }
    import_jobs::update_job_progress(conn, job.id, 0, total_rows, 0)?;

    // Phase 1: Parse and validate ALL rows — fail fast on any error
    let mut parsed_rows: Vec<serde_json::Map<String, serde_json::Value>> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (i, row) in rows.iter().skip(1).enumerate() {
        let row_num = i + 1;
        let mut obj = serde_json::Map::new();
        let mut row_errors = Vec::new();

        for (j, cell) in row.iter().enumerate() {
            let key = header_names.get(j).cloned().unwrap_or_else(|| format!("col_{}", j));
            let val = cell.to_string().trim().to_string();
            obj.insert(key.clone(), serde_json::Value::String(val.clone()));

            if key == "item_name" && val.is_empty() {
                row_errors.push("missing required field 'item_name'".to_string());
            }
            if key == "quantity_on_hand" && val.parse::<i32>().is_err() {
                row_errors.push(format!("invalid integer for 'quantity_on_hand': '{}'", val));
            }
        }

        if !row_errors.is_empty() {
            errors.push(format!("Row {}: {}", row_num, row_errors.join("; ")));
        }
        parsed_rows.push(obj);
    }

    // Fail fast: if ANY row has errors, abort before touching the target table
    if !errors.is_empty() {
        let log = errors.join("\n");
        import_jobs::mark_job_failed(conn, job.id, &log)?;
        return Err(format!("{} of {} rows failed validation:\n{}", errors.len(), total_rows, log).into());
    }

    // Phase 2: Atomic staging-to-target commit
    let row_count = parsed_rows.len() as i32;
    let staging = format!("_staging_{}", job.id.to_string().replace('-', "_"));

    conn.transaction(|tx| {
        // Create staging table matching target schema
        diesel::sql_query(format!(
            "CREATE TEMP TABLE {} (\
             facility_id UUID, warehouse_id UUID, bin_id UUID, \
             item_name TEXT, lot_number TEXT, quantity_on_hand INT\
             )", staging
        )).execute(tx)?;

        // Insert validated rows into staging
        for obj in &parsed_rows {
            let item   = obj.get("item_name").and_then(|v| v.as_str()).unwrap_or("");
            let lot    = obj.get("lot_number").and_then(|v| v.as_str()).unwrap_or("IMPORTED");
            let qty: i32 = obj.get("quantity_on_hand").and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()).unwrap_or(0);
            let fid = obj.get("facility_id").and_then(|v| v.as_str())
                .unwrap_or("00000000-0000-0000-0000-000000000001");
            let wid = obj.get("warehouse_id").and_then(|v| v.as_str())
                .unwrap_or("00000000-0000-0000-0000-000000000002");
            let bid = obj.get("bin_id").and_then(|v| v.as_str())
                .unwrap_or("00000000-0000-0000-0000-000000000003");

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
            .execute(tx)?;
        }

        // Single INSERT...SELECT from staging to target
        diesel::sql_query(format!(
            "INSERT INTO inventory_lots \
             (id, facility_id, warehouse_id, bin_id, item_name, lot_number, \
              quantity_on_hand, quantity_reserved, created_at, updated_at) \
             SELECT gen_random_uuid(), facility_id, warehouse_id, bin_id, \
                    item_name, lot_number, quantity_on_hand, 0, now(), now() \
             FROM {}", staging
        )).execute(tx)?;

        // Clean up staging
        diesel::sql_query(format!("DROP TABLE {}", staging)).execute(tx)?;

        // Audit entry
        diesel::sql_query(format!(
            "INSERT INTO audit_log (id, actor_id, action, entity_type, detail, created_at) \
             VALUES (gen_random_uuid(), '{}', 'bulk_import', 'inventory_lots', \
             '{{}}'::jsonb, now())", job.created_by
        )).execute(tx)?;

        Ok::<_, diesel::result::Error>(())
    }).map_err(|e| {
        tracing::error!(job_id = %job.id, error = %e, "Import transaction rolled back");
        format!("Transaction rolled back: {}", e)
    })?;

    import_jobs::update_job_progress(conn, job.id, row_count, total_rows, 100)?;
    Ok(())
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
        "UPDATE resources SET state = 'published', updated_at = now() \
         WHERE scheduled_publish_at <= $1 \
         AND scheduled_publish_at IS NOT NULL \
         AND state = 'in_review'"
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)?;

    if count > 0 {
        tracing::info!(count = count, "Published {} scheduled resources", count);
    }
    Ok(())
}
