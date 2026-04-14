use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use prometheus::{Encoder, TextEncoder};

use crate::errors::ApiError;
use crate::middleware::auth_guard::RbacContext;
use crate::repository::{import_jobs, sessions};
use crate::AppState;

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    cnt: i64,
}

/// Returns Prometheus-formatted metrics including sessions, job queue depth, and uptime.
pub async fn prometheus_metrics(
    state: web::Data<AppState>,
    _ctx: RbacContext,
) -> Result<HttpResponse, ApiError> {
    let mut conn = state.db_pool.get()?;

    let active_sessions = sessions::count_active_sessions(&mut conn).unwrap_or(0);
    let queue_depth = import_jobs::count_queued(&mut conn).unwrap_or(0);
    let uptime = state.start_time.elapsed().as_secs();

    // Real metrics from DB
    let completed_imports: i64 = diesel::sql_query(
        "SELECT COUNT(*) as cnt FROM import_jobs WHERE status = 'completed'"
    ).get_result::<CountRow>(&mut conn).map(|r| r.cnt).unwrap_or(0);

    let failed_imports: i64 = diesel::sql_query(
        "SELECT COUNT(*) as cnt FROM import_jobs WHERE status = 'failed'"
    ).get_result::<CountRow>(&mut conn).map(|r| r.cnt).unwrap_or(0);

    let scheduled_published: i64 = diesel::sql_query(
        "SELECT COUNT(*) as cnt FROM resources WHERE state = 'published' AND scheduled_publish_at IS NOT NULL"
    ).get_result::<CountRow>(&mut conn).map(|r| r.cnt).unwrap_or(0);

    let body = format!(
        "# HELP tourism_active_sessions Number of active sessions\n\
         # TYPE tourism_active_sessions gauge\n\
         tourism_active_sessions {}\n\
         # HELP tourism_job_queue_depth Number of queued/running import jobs\n\
         # TYPE tourism_job_queue_depth gauge\n\
         tourism_job_queue_depth {}\n\
         # HELP tourism_uptime_seconds Server uptime in seconds\n\
         # TYPE tourism_uptime_seconds gauge\n\
         tourism_uptime_seconds {}\n\
         # HELP tourism_import_completed_total Total completed import jobs\n\
         # TYPE tourism_import_completed_total counter\n\
         tourism_import_completed_total {}\n\
         # HELP tourism_import_failed_total Total failed import jobs\n\
         # TYPE tourism_import_failed_total counter\n\
         tourism_import_failed_total {}\n\
         # HELP tourism_scheduled_published_total Resources auto-published via scheduler\n\
         # TYPE tourism_scheduled_published_total counter\n\
         tourism_scheduled_published_total {}\n",
        active_sessions, queue_depth, uptime,
        completed_imports, failed_imports, scheduled_published
    );

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).ok();
    let registry_metrics = String::from_utf8(buffer).unwrap_or_default();

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(format!("{}{}", body, registry_metrics)))
}
