//! Response-schema assertions for RBAC and security error responses.
//!
//! Every 403/401 response from the backend must conform to ApiErrorBody:
//!   { "code": String, "message": String, "details": [] }
//!
//! This module verifies the response body schema in addition to the HTTP status,
//! ensuring the frontend's error-display components always receive a well-formed
//! payload.

use crate::helpers::*;

/// Helper: parse a 403 response body and assert it matches ApiErrorBody schema.
async fn assert_forbidden_body(resp: reqwest::Response) {
    assert_eq!(resp.status().as_u16(), 403);
    let body: serde_json::Value = resp.json().await.expect("403 body must be JSON");
    assert!(
        body.get("code").and_then(|v| v.as_str()).is_some(),
        "403 body must have a string 'code' field; got: {body}"
    );
    assert_eq!(
        body["code"].as_str().unwrap(),
        "FORBIDDEN",
        "403 code must be 'FORBIDDEN'; got: {body}"
    );
    assert!(
        body.get("message").and_then(|v| v.as_str()).is_some(),
        "403 body must have a string 'message' field; got: {body}"
    );
    // 'details' may be absent (backend uses skip_serializing_if = "Vec::is_empty")
    // but if present it must be an array.
    if let Some(details) = body.get("details") {
        assert!(details.is_array(), "'details' must be an array; got: {body}");
    }
}

/// Helper: parse a 401 response body and assert it matches ApiErrorBody schema.
async fn assert_unauthorized_body(resp: reqwest::Response) {
    assert_eq!(resp.status().as_u16(), 401);
    let body: serde_json::Value = resp.json().await.expect("401 body must be JSON");
    assert!(
        body.get("code").and_then(|v| v.as_str()).is_some(),
        "401 body must have a string 'code' field; got: {body}"
    );
    assert_eq!(
        body["code"].as_str().unwrap(),
        "UNAUTHORIZED",
        "401 code must be 'UNAUTHORIZED'; got: {body}"
    );
    assert!(
        body.get("message").and_then(|v| v.as_str()).is_some(),
        "401 body must have a string 'message' field; got: {body}"
    );
}

// ── 403 body schema: RBAC role denials ────────────────────────────────────────

#[tokio::test]
async fn clinician_create_lodging_forbidden_body_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clinician").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"name": "T", "amenities": []}))
        .send().await.unwrap();

    assert_forbidden_body(resp).await;
}

#[tokio::test]
async fn clerk_create_resource_forbidden_body_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "T", "address": "A", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();

    assert_forbidden_body(resp).await;
}

#[tokio::test]
async fn reviewer_create_resource_forbidden_body_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "T", "address": "A", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();

    assert_forbidden_body(resp).await;
}

#[tokio::test]
async fn publisher_view_inventory_forbidden_body_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "publisher").await;
    let c = bearer_client(&session);

    let resp = c.get(&format!("{}/api/inventory/lots", base_url()))
        .send().await.unwrap();

    assert_forbidden_body(resp).await;
}

// ── 403 body schema: CSRF enforcement ─────────────────────────────────────────

#[tokio::test]
async fn missing_csrf_forbidden_body_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .json(&serde_json::json!({
            "title": "No CSRF", "address": "123 St", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();

    assert_forbidden_body(resp).await;
}

#[tokio::test]
async fn wrong_csrf_forbidden_body_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", "bogus-value")
        .json(&serde_json::json!({
            "title": "Bad CSRF", "address": "789 Rd", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();

    assert_forbidden_body(resp).await;
}

// ── 401 body schema: unauthenticated requests ─────────────────────────────────

#[tokio::test]
async fn unauthenticated_request_returns_401_with_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = authed_client();

    let resp = c.get(&format!("{}/api/resources", base_url()))
        .send().await.unwrap();

    assert_unauthorized_body(resp).await;
}

#[tokio::test]
async fn invalid_bearer_token_returns_401_with_schema() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = bearer_client("totally-invalid-token");

    let resp = c.get(&format!("{}/api/resources", base_url()))
        .send().await.unwrap();

    assert_unauthorized_body(resp).await;
}
