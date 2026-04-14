//! Tests for the Configuration Center endpoints:
//! GET /api/config, POST /api/config, GET /api/config/:key
//!
//! All calls are real external HTTP calls through base_url().
//! Administrator-only: non-admin roles must receive 403.

use crate::helpers::*;

// ── List config parameters ────────────────────────────────────────────────────

/// GET /api/config returns a JSON array for Administrators.
#[tokio::test]
async fn config_list_returns_array_for_admin() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!("{}/api/config", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array(), "config list must return a JSON array");
}

/// Non-administrator roles must receive 403 on GET /api/config.
#[tokio::test]
async fn config_list_blocked_for_non_admin_roles() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    for username in &["publisher", "reviewer", "clinician", "clerk"] {
        let (session, _csrf) = login_as(&authed_client(), username).await;
        let c = bearer_client(&session);

        let resp = c
            .get(&format!("{}/api/config", base))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            403,
            "GET /api/config must be blocked for role {} (got {})",
            username,
            resp.status()
        );
    }
}

/// GET /api/config requires authentication.
#[tokio::test]
async fn config_list_requires_authentication() {
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .get(&format!("{}/api/config", base_url()))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "GET /api/config must require auth, got {}",
        resp.status()
    );
}

// ── Upsert config parameter ───────────────────────────────────────────────────

/// POST /api/config creates a new parameter and returns it.
#[tokio::test]
async fn config_upsert_creates_new_parameter() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/config", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "key": "test.feature.threshold",
            "value": "42",
            "feature_switch": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "upsert must return 200");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["key"], "test.feature.threshold");
    assert_eq!(body["value"], "42");
}

/// POST /api/config updates an existing parameter (idempotent upsert).
#[tokio::test]
async fn config_upsert_updates_existing_parameter() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // First write
    c.post(&format!("{}/api/config", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "key": "test.upsert.key",
            "value": "initial",
            "feature_switch": false
        }))
        .send()
        .await
        .unwrap();

    // Second write with a new value
    let resp = c
        .post(&format!("{}/api/config", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "key": "test.upsert.key",
            "value": "updated",
            "feature_switch": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["value"], "updated", "upsert must overwrite the previous value");
}

/// POST /api/config with feature_switch=true is persisted correctly.
#[tokio::test]
async fn config_upsert_persists_feature_switch_flag() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/config", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "key": "test.feature.switch",
            "value": "true",
            "feature_switch": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["feature_switch"], true);
}

/// Non-administrator roles must receive 403 on POST /api/config.
#[tokio::test]
async fn config_upsert_blocked_for_non_admin_roles() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    for username in &["publisher", "reviewer", "clinician", "clerk"] {
        let (session, csrf) = login_as(&authed_client(), username).await;
        let c = bearer_client(&session);

        let resp = c
            .post(&format!("{}/api/config", base))
            .header("X-CSRF-Token", csrf)
            .json(&serde_json::json!({
                "key": "should.not.be.written",
                "value": "blocked",
                "feature_switch": false
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            403,
            "POST /api/config must be blocked for role {} (got {})",
            username,
            resp.status()
        );
    }
}

// ── Get single config parameter ───────────────────────────────────────────────

/// GET /api/config/:key returns the parameter for an Administrator.
#[tokio::test]
async fn config_get_by_key_returns_parameter() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Seed the key first
    c.post(&format!("{}/api/config", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "key": "test.get.by.key",
            "value": "hello",
            "feature_switch": false
        }))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(&format!("{}/api/config/test.get.by.key", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["key"], "test.get.by.key");
    assert_eq!(body["value"], "hello");
}

/// GET /api/config/:key returns 404 for an unknown key.
#[tokio::test]
async fn config_get_by_key_returns_404_for_missing_key() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!("{}/api/config/nonexistent.key.xyz", base))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        404,
        "unknown config key must return 404"
    );
}

/// GET /api/config/:key is restricted to Administrators.
#[tokio::test]
async fn config_get_by_key_blocked_for_non_admin_roles() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // Seed the key as admin first
    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    bearer_client(&admin_session)
        .post(&format!("{}/api/config", base))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({
            "key": "test.rbac.key",
            "value": "secret",
            "feature_switch": false
        }))
        .send()
        .await
        .unwrap();

    for username in &["publisher", "reviewer", "clinician", "clerk"] {
        let (session, _csrf) = login_as(&authed_client(), username).await;
        let c = bearer_client(&session);

        let resp = c
            .get(&format!("{}/api/config/test.rbac.key", base))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            403,
            "GET /api/config/:key must be blocked for role {} (got {})",
            username,
            resp.status()
        );
    }
}

// ── Round-trip integrity ──────────────────────────────────────────────────────

/// Values written via POST must be readable via GET and appear in the list.
#[tokio::test]
async fn config_round_trip_write_and_read() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let key = "test.round.trip";
    let value = "round-trip-value-1234";

    // Write
    c.post(&format!("{}/api/config", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "key": key,
            "value": value,
            "feature_switch": false
        }))
        .send()
        .await
        .unwrap();

    // Read by key
    let resp = c
        .get(&format!("{}/api/config/{}", base, key))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let single: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(single["value"], value);

    // Read via list
    let resp = c
        .get(&format!("{}/api/config", base))
        .send()
        .await
        .unwrap();
    let list: serde_json::Value = resp.json().await.unwrap();
    let found = list
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["key"] == key && p["value"] == value);
    assert!(found, "written key must appear in the config list");
}
