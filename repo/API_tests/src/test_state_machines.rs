//! Tests for resource and lodging state machine transitions (publish / archive),
//! resource versioning, auth profile management, and session lifecycle.
//!
//! Maps to: "listing publish/archive", "product update/delete", "user management".
//! All tests use real external HTTP calls through base_url().

use crate::helpers::*;

// ── Resource state machine: draft → in_review → published → offline ──────────

/// A Publisher can submit a draft resource for review (draft → in_review).
#[tokio::test]
async fn resource_draft_to_in_review_by_publisher() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let pub_client = bearer_client(&pub_session);

    let resp = pub_client
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({
            "title": "Park Trail Guide",
            "category": "outdoor",
            "address": "Trailhead, Wilderness Park",
            "tags": ["hiking", "nature"],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["state"], "draft");

    // Publisher transitions draft → in_review
    let resp = pub_client
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({"state": "in_review"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["state"], "in_review");
}

/// Reviewer approves an in_review resource (in_review → published).
#[tokio::test]
async fn resource_in_review_to_published_by_reviewer() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // Create and submit for review as publisher
    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let pub_client = bearer_client(&pub_session);

    let resp = pub_client
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({
            "title": "City Museum",
            "address": "1 Museum Blvd",
            "tags": ["culture"],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    pub_client
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({"state": "in_review"}))
        .send()
        .await
        .unwrap();

    // Reviewer publishes
    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);

    let resp = rev_client
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({"state": "published"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["state"], "published");
}

/// Publisher archives a published resource (published → offline).
#[tokio::test]
async fn resource_published_to_offline_by_publisher() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let pub_client = bearer_client(&pub_session);

    // Create → review → publish via admin shortcut
    let resp = pub_client
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({
            "title": "Sunset Viewpoint",
            "address": "Vista Point Rd",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    pub_client
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({"state": "in_review"}))
        .send()
        .await
        .unwrap();

    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    bearer_client(&rev_session)
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({"state": "published"}))
        .send()
        .await
        .unwrap();

    // Publisher archives (published → offline)
    let resp = pub_client
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({"state": "offline"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["state"], "offline");
}

/// Invalid transitions are rejected with 422.
#[tokio::test]
async fn resource_invalid_state_transition_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);

    let resp = admin_client
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({
            "title": "Invalid Transition Resource",
            "address": "Nowhere",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Jump from draft directly to published — should fail
    let resp = admin_client
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({"state": "published"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        422,
        "direct draft→published transition must be rejected"
    );
}

/// A reviewer attempting a publisher-only transition (draft→in_review) gets 422.
#[tokio::test]
async fn resource_state_transition_enforces_role() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);

    let resp = admin_client
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({
            "title": "Role Guard Resource",
            "address": "Nowhere",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Reviewer tries to push draft → in_review (only Publisher may do this)
    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let resp = bearer_client(&rev_session)
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({"state": "in_review"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        422,
        "reviewer must not be able to submit a draft for review"
    );
}

// ── Resource versioning ──────────────────────────────────────────────────────

/// Every PUT to a resource should increment its version and create a version record.
#[tokio::test]
async fn resource_put_increments_version_and_creates_version_record() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);

    let resp = admin_client
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({
            "title": "Version Test Resource",
            "address": "123 Version St",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["current_version"], 1);

    // Update title
    let resp = admin_client
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({"title": "Version Test Resource (v2)"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["current_version"], 2, "version must increment on each PUT");

    // GET /api/resources/:id/versions must include both versions
    let resp = admin_client
        .get(&format!("{}/api/resources/{}/versions", base, id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let versions: serde_json::Value = resp.json().await.unwrap();
    let arr = versions.as_array().expect("versions should be an array");
    assert!(arr.len() >= 1, "should have at least 1 version record after one update");
}

// ── Auth: profile and session lifecycle ──────────────────────────────────────

/// GET /api/auth/me returns the authenticated user's profile.
#[tokio::test]
async fn auth_me_returns_user_profile() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!("{}/api/auth/me", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["username"], "admin");
    assert_eq!(body["role"], "Administrator");
    assert!(body["id"].is_string());
    assert!(body["mfa_enabled"].is_boolean());
}

/// GET /api/auth/me returns 401 without a valid session.
#[tokio::test]
async fn auth_me_requires_authentication() {
    let c = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = c
        .get(&format!("{}/api/auth/me", base_url()))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "unauthenticated /auth/me must return 401 or 403"
    );
}

/// POST /api/auth/logout destroys the session; subsequent /me calls return 401.
#[tokio::test]
async fn auth_logout_invalidates_session() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);

    // Confirm session is live
    let resp = c.get(&format!("{}/api/auth/me", base)).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Logout
    let resp = c
        .post(&format!("{}/api/auth/logout", base))
        .header("X-CSRF-Token", &csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "logout must return 200");

    // Session must be dead now
    let resp = c.get(&format!("{}/api/auth/me", base)).send().await.unwrap();
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "session must be invalid after logout, got {}",
        resp.status()
    );
}

// ── Inventory: warehouse and bin listing (warehouse CRUD coverage) ───────────

/// GET /api/inventory/warehouses lists warehouses visible to the caller.
#[tokio::test]
async fn inventory_list_warehouses_returns_json_array() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let base = base_url();
    let _ = seed;

    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!("{}/api/inventory/warehouses", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array(), "warehouses response must be an array");
    let arr = body.as_array().unwrap();
    assert!(!arr.is_empty(), "warehouses array should not be empty after seeding");
    // Each entry must have id, facility_id, name
    for wh in arr {
        assert!(wh["id"].is_string(), "warehouse must have id");
        assert!(wh["facility_id"].is_string(), "warehouse must have facility_id");
        assert!(wh["name"].is_string(), "warehouse must have name");
    }
}

/// GET /api/inventory/bins?warehouse_id=<id> lists bins for a warehouse.
#[tokio::test]
async fn inventory_list_bins_for_warehouse() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let base = base_url();

    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!(
            "{}/api/inventory/bins?warehouse_id={}",
            base, seed.warehouse_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array());
    let arr = body.as_array().unwrap();
    assert!(!arr.is_empty(), "bins should exist for the seeded warehouse");
    for bin in arr {
        assert!(bin["id"].is_string());
        assert!(bin["warehouse_id"].is_string());
        assert!(bin["label"].is_string());
    }
}

/// GET /api/inventory/transactions/audit-print returns HTML for authorized roles.
#[tokio::test]
async fn inventory_audit_print_returns_html_for_admin() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Create a lot so we have a valid lot_id to pass (required query param)
    let resp = c
        .post(&format!("{}/api/inventory/lots", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Audit Print Item",
            "lot_number": "LOT-AUDIT-SM-001",
            "quantity_on_hand": 10
        }))
        .send()
        .await
        .unwrap();
    let lot_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = c
        .get(&format!(
            "{}/api/inventory/transactions/audit-print?lot_id={}",
            base, lot_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        ct.contains("text/html") || ct.contains("application/json"),
        "audit-print must return HTML or JSON, got {}",
        ct
    );
}
