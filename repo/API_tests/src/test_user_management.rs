//! Tests for user management workflows:
//! MFA enrollment (setup → confirm → disable), profile retrieval,
//! category-filtered resource listing, and role-based access patterns.
//!
//! "Benefit redemption" in this domain maps to the two-party export-approval
//! flow: the requesting user cannot approve their own export (second-person gate).

use crate::helpers::*;

// ── MFA enrollment: setup ────────────────────────────────────────────────────

/// GET /api/auth/mfa/setup returns a TOTP provisioning payload for any
/// authenticated user, regardless of role.
#[tokio::test]
async fn mfa_setup_returns_totp_provisioning_payload() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    for username in &["admin", "publisher", "reviewer", "clinician", "clerk"] {
        let (session, _csrf) = login_as(&authed_client(), username).await;
        let c = bearer_client(&session);

        let resp = c
            .get(&format!("{}/api/auth/mfa/setup", base))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            200,
            "mfa/setup must be accessible to role {} (got {})",
            username,
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["secret_base64"].is_string(), "secret_base64 must be present");
        assert!(body["issuer"].is_string(), "issuer must be present");
        assert!(body["digits"].is_number(), "digits must be present");
        assert!(body["period"].is_number(), "period must be present");
        // Secret must be non-empty base64
        let secret = body["secret_base64"].as_str().unwrap();
        assert!(!secret.is_empty(), "secret_base64 must not be empty");
    }
}

/// GET /api/auth/mfa/setup requires authentication.
#[tokio::test]
async fn mfa_setup_requires_authentication() {
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .get(&format!("{}/api/auth/mfa/setup", base_url()))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "mfa/setup must reject unauthenticated callers (got {})",
        resp.status()
    );
}

/// Each call to GET /api/auth/mfa/setup must return a fresh secret
/// (secrets must not be reused across provisioning sessions).
#[tokio::test]
async fn mfa_setup_returns_fresh_secret_each_call() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let r1: serde_json::Value = c
        .get(&format!("{}/api/auth/mfa/setup", base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let r2: serde_json::Value = c
        .get(&format!("{}/api/auth/mfa/setup", base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_ne!(
        r1["secret_base64"], r2["secret_base64"],
        "successive setup calls must return different secrets"
    );
}

// ── MFA confirm: validation errors ───────────────────────────────────────────

/// POST /api/auth/mfa/confirm with a missing secret_base64 field returns 400.
#[tokio::test]
async fn mfa_confirm_missing_secret_returns_400() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/auth/mfa/confirm", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"code": "123456"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        400,
        "confirm without secret_base64 must return 400"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "MISSING_FIELD");
}

/// POST /api/auth/mfa/confirm with a missing TOTP code field returns 400.
#[tokio::test]
async fn mfa_confirm_missing_code_returns_400() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/auth/mfa/confirm", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"secret_base64": "YWJj"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        400,
        "confirm without code must return 400"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "MISSING_FIELD");
}

/// POST /api/auth/mfa/confirm with an invalid TOTP code returns 401.
#[tokio::test]
async fn mfa_confirm_invalid_code_returns_401() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Obtain a real secret from setup
    let setup: serde_json::Value = c
        .get(&format!("{}/api/auth/mfa/setup", base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let secret = setup["secret_base64"].as_str().unwrap();

    let resp = c
        .post(&format!("{}/api/auth/mfa/confirm", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "secret_base64": secret,
            "code": "000000"
        }))
        .send()
        .await
        .unwrap();
    // An obviously wrong code must be rejected
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "confirm with bad code must return 401/403, got {}",
        resp.status()
    );
}

/// POST /api/auth/mfa/confirm with non-base64 secret returns 400.
#[tokio::test]
async fn mfa_confirm_invalid_base64_secret_returns_400() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/auth/mfa/confirm", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "secret_base64": "!!not-valid-base64!!",
            "code": "123456"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        400,
        "confirm with invalid base64 must return 400, got {}",
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "INVALID_SECRET");
}

// ── MFA disable: validation errors ───────────────────────────────────────────

/// POST /api/auth/mfa/disable when MFA is not enabled returns a 400 with
/// MFA_NOT_ENABLED code.
#[tokio::test]
async fn mfa_disable_when_not_enabled_returns_400() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // The seeded "reviewer" account has mfa_enabled = false
    let (session, csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/auth/mfa/disable", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"code": "123456"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        400,
        "disable on non-MFA account must return 400, got {}",
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "MFA_NOT_ENABLED");
}

/// POST /api/auth/mfa/disable with a missing code field returns 400.
#[tokio::test]
async fn mfa_disable_missing_code_returns_400() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/auth/mfa/disable", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        400,
        "disable without code must return 400, got {}",
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "MISSING_FIELD");
}

/// GET /api/auth/mfa/setup and POST /api/auth/mfa/confirm both require auth.
#[tokio::test]
async fn mfa_confirm_requires_authentication() {
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .post(&format!("{}/api/auth/mfa/confirm", base_url()))
        .json(&serde_json::json!({"secret_base64": "test", "code": "123456"}))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "mfa/confirm must reject unauthenticated callers, got {}",
        resp.status()
    );
}

// ── Category-filtered resource listing ───────────────────────────────────────

/// GET /api/resources?category=<x> returns only resources matching that category.
#[tokio::test]
async fn resource_list_filtered_by_category() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Create two resources with distinct categories
    c.post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Hotel Sunrise",
            "category": "hotel",
            "address": "1 Main St",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();

    c.post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Nature Trail",
            "category": "outdoor",
            "address": "2 Trail Rd",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(&format!("{}/api/resources?category=hotel", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let items = body["data"].as_array().expect("data array");
    assert!(
        items.iter().all(|r| r["category"] == "hotel"),
        "category filter must return only hotel resources"
    );
}

/// GET /api/resources?state=draft returns only draft-state resources.
#[tokio::test]
async fn resource_list_filtered_by_state() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    c.post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Draft Resource",
            "address": "1 Draft St",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(&format!("{}/api/resources?state=draft", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let items = body["data"].as_array().expect("data array");
    assert!(
        items.iter().all(|r| r["state"] == "draft"),
        "state filter must return only draft resources"
    );
}

// ── Benefit redemption: second-person export approval ────────────────────────

/// The user who requested an export must not be able to approve their own
/// export (second-person approval gate). A different user must approve.
#[tokio::test]
async fn export_requester_cannot_approve_own_export() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // Reviewer creates the export request
    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);

    let resp = rev_client
        .post(&format!("{}/api/export/request", base))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({"export_type": "inventory"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    let export_id = body["id"].as_str().unwrap().to_string();

    // Same reviewer tries to approve — should be blocked
    let resp = rev_client
        .post(&format!("{}/api/export/approve/{}", base, export_id))
        .header("X-CSRF-Token", &rev_csrf)
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == 403 || resp.status() == 422,
        "requester must not be able to approve their own export, got {}",
        resp.status()
    );
}

/// GET /api/export/pending returns a list of pending exports for authorized users.
#[tokio::test]
async fn export_pending_list_returns_array() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);

    // Create a pending export
    c.post(&format!("{}/api/export/request", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"export_type": "resources"}))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(&format!("{}/api/export/pending", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array(), "pending exports must be a JSON array");
}
