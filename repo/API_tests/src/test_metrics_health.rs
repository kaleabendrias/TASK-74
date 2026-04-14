use crate::helpers::*;

// ── /api/health — public liveness probe ──────────────────────────────────────

#[tokio::test]
async fn health_liveness_is_public_and_returns_200() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    // No auth header — liveness is intentionally unauthenticated
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .get(&format!("{}/api/health", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    // Liveness returns only {"status": "ok"/"degraded"} — no sensitive fields
    assert!(body["status"].is_string(), "status field must be present");
    let status = body["status"].as_str().unwrap();
    assert!(
        status == "ok" || status == "degraded",
        "unexpected liveness status: {}",
        status
    );
}

#[tokio::test]
async fn health_liveness_does_not_expose_internal_fields() {
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .get(&format!("{}/api/health", base_url()))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    // Internal fields must NOT appear in the public probe
    assert!(body.get("version").is_none(), "/api/health must not expose version");
    assert!(body.get("config_profile").is_none(), "/api/health must not expose config_profile");
    assert!(body.get("disk_usage_bytes").is_none(), "/api/health must not expose disk usage");
    assert!(body.get("service").is_none(), "/api/health must not expose service name");
}

// ── /api/health/ready — protected readiness probe ───────────────────────────

#[tokio::test]
async fn health_readiness_requires_authentication() {
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .get(&format!("{}/api/health/ready", base_url()))
        .send()
        .await
        .unwrap();
    // Must reject unauthenticated callers — 401 or 403
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "readiness probe must require auth, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn health_readiness_returns_full_details_when_authenticated() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!("{}/api/health/ready", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["service"].is_string(), "readiness must include service name");
    assert!(body["version"].is_string(), "readiness must include version");
    assert!(body["uptime_secs"].is_number(), "readiness must include uptime");
    assert_eq!(body["database_connected"], true, "database must be connected");
    assert!(body["config_profile"].is_string(), "readiness must include config_profile");
}

#[tokio::test]
async fn health_readiness_accessible_to_any_authenticated_role() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    for username in &["reviewer", "publisher", "clinician", "clerk"] {
        let (session, _csrf) = login_as(&authed_client(), username).await;
        let c = bearer_client(&session);
        let resp = c
            .get(&format!("{}/api/health/ready", base_url()))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            200,
            "readiness should be accessible to all authenticated users (failed for {})",
            username
        );
    }
}

// ── /api/metrics — Administrator-only ────────────────────────────────────────

#[tokio::test]
async fn metrics_returns_prometheus_format_for_admin() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!("{}/api/metrics", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("text/plain"));
    let body = resp.text().await.unwrap();
    assert!(body.contains("tourism_active_sessions"));
    assert!(body.contains("tourism_job_queue_depth"));
    assert!(body.contains("tourism_uptime_seconds"));
    assert!(body.contains("tourism_import_completed_total"));
    assert!(body.contains("tourism_import_failed_total"));
    assert!(body.contains("tourism_scheduled_published_total"));
}

#[tokio::test]
async fn metrics_blocked_for_non_administrator_roles() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    for username in &["reviewer", "publisher", "clinician", "clerk"] {
        let (session, _csrf) = login_as(&authed_client(), username).await;
        let c = bearer_client(&session);
        let resp = c
            .get(&format!("{}/api/metrics", base_url()))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            403,
            "metrics endpoint must be blocked for role {} (got {})",
            username,
            resp.status()
        );
    }
}

#[tokio::test]
async fn metrics_blocked_without_authentication() {
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .get(&format!("{}/api/metrics", base_url()))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "metrics must require auth, got {}",
        resp.status()
    );
}
