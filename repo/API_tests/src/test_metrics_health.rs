use crate::helpers::*;

#[tokio::test]
async fn health_returns_200() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();

    let resp = c.get(&format!("{}/api/health", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["database_connected"], true);
    assert!(body["service"].is_string());
    assert!(body["version"].is_string());
    assert!(body["uptime_secs"].is_number());
    assert!(body["config_profile"].is_string());
}

#[tokio::test]
async fn metrics_returns_prometheus_format() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.get(&format!("{}/api/metrics", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("text/plain"));
    let body = resp.text().await.unwrap();
    assert!(body.contains("tourism_active_sessions"));
    assert!(body.contains("tourism_job_queue_depth"));
    assert!(body.contains("tourism_uptime_seconds"));
    assert!(body.contains("tourism_request_count_total"));
    assert!(body.contains("tourism_errors_total"));
}

#[tokio::test]
async fn health_has_correct_fields() {
    let c = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
    let resp = c.get(&format!("{}/api/health", base_url()))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    // All required fields present
    assert!(body.get("service").is_some());
    assert!(body.get("version").is_some());
    assert!(body.get("uptime_secs").is_some());
    assert!(body.get("database_connected").is_some());
    assert!(body.get("config_profile").is_some());
}
