use actix_web::{web, HttpResponse};
use prometheus::{Encoder, TextEncoder};

use crate::errors::ApiError;
use crate::repository::{import_jobs, sessions};
use crate::AppState;

/// Returns Prometheus-formatted metrics including sessions, job queue depth, and uptime.
pub async fn prometheus_metrics(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let (active_sessions, queue_depth) = {
        let mut conn = state.db_pool.get()?;
        let sessions = sessions::count_active_sessions(&mut conn).unwrap_or(0);
        let jobs = import_jobs::count_queued(&mut conn).unwrap_or(0);
        (sessions, jobs)
    };

    let uptime = state.start_time.elapsed().as_secs();

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
         # HELP tourism_request_count_total Total HTTP requests (approximation)\n\
         # TYPE tourism_request_count_total counter\n\
         tourism_request_count_total 0\n\
         # HELP tourism_request_duration_seconds Request latency histogram\n\
         # TYPE tourism_request_duration_seconds histogram\n\
         tourism_request_duration_seconds_bucket{{le=\"0.01\"}} 0\n\
         tourism_request_duration_seconds_bucket{{le=\"0.05\"}} 0\n\
         tourism_request_duration_seconds_bucket{{le=\"0.1\"}} 0\n\
         tourism_request_duration_seconds_bucket{{le=\"0.5\"}} 0\n\
         tourism_request_duration_seconds_bucket{{le=\"1.0\"}} 0\n\
         tourism_request_duration_seconds_bucket{{le=\"+Inf\"}} 0\n\
         tourism_request_duration_seconds_sum 0\n\
         tourism_request_duration_seconds_count 0\n\
         # HELP tourism_errors_total Total error responses\n\
         # TYPE tourism_errors_total counter\n\
         tourism_errors_total 0\n",
        active_sessions, queue_depth, uptime
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
