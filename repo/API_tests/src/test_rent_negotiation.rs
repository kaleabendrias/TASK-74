//! Tests for the rent-change negotiation workflow:
//! request (pending) → counterpropose (countered) → accept-counter (approved).
//!
//! All calls are real external HTTP calls routed through the base_url() helper,
//! which honours the TEST_BASE_URL env-var (defaults to localhost).
//! No mocks or in-process simulations are used.

use crate::helpers::*;

// ── Shared helper ────────────────────────────────────────────────────────────

/// Creates a lodging and returns its ID.
async fn create_lodging(c: &reqwest::Client, csrf: &str, base: &str) -> String {
    let resp = c
        .post(&format!("{}/api/lodgings", base))
        .header("X-CSRF-Token", csrf)
        .json(&serde_json::json!({
            "name": "Negotiation Suite",
            "amenities": ["wifi"],
            "monthly_rent": 2000.0,
            "deposit_amount": 2000.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "failed to create lodging for test setup");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

/// Creates a pending rent-change request and returns its ID.
async fn create_rent_change(c: &reqwest::Client, csrf: &str, base: &str, lodging_id: &str) -> String {
    let resp = c
        .put(&format!("{}/api/lodgings/{}/rent-change", base, lodging_id))
        .header("X-CSRF-Token", csrf)
        .json(&serde_json::json!({
            "proposed_rent": 2200.0,
            "proposed_deposit": 2200.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "failed to create rent change for test setup");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

// ── Full negotiation flow ────────────────────────────────────────────────────

/// Happy-path negotiation: Publisher requests → Reviewer counters → Publisher
/// accepts the counter, which applies the counterproposed values to the lodging.
#[tokio::test]
async fn rent_change_full_negotiation_flow() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // 1. Publisher creates lodging + rent-change request
    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let pub_client = bearer_client(&pub_session);
    let lodging_id = create_lodging(&pub_client, &pub_csrf, &base).await;
    let change_id = create_rent_change(&pub_client, &pub_csrf, &base, &lodging_id).await;

    // Verify initial status is "pending" — pending list requires Reviewer/Admin role
    let (rev_session_early, _) = login_as(&authed_client(), "reviewer").await;
    let resp = bearer_client(&rev_session_early)
        .get(&format!("{}/api/lodgings/rent-changes/pending", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert!(list.as_array().unwrap().iter().any(|r| r["id"] == change_id));

    // 2. Reviewer submits a counterproposal
    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);

    let resp = rev_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/counterpropose",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({
            "proposed_rent": 2100.0,
            "proposed_deposit": 2100.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "counterpropose should return 200");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "countered", "status must be 'countered' after counterproposal");
    assert_eq!(body["counterproposal_rent"], 2100.0);
    assert_eq!(body["counterproposal_deposit"], 2100.0);
    assert!(body["counterproposed_by"].is_string());
    assert!(body["counterproposed_at"].is_string());

    // The pending list must now include the countered change
    let resp = rev_client
        .get(&format!("{}/api/lodgings/rent-changes/pending", base))
        .send()
        .await
        .unwrap();
    let list: serde_json::Value = resp.json().await.unwrap();
    let found = list
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == change_id)
        .cloned()
        .expect("countered change must appear in pending list");
    assert_eq!(found["status"], "countered");

    // 3. Publisher accepts the counter — this applies counterproposed values to the lodging
    let resp = pub_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/accept-counter",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &pub_csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "accept-counter should return 200");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "approved", "status must be 'approved' after accepting counter");

    // 4. Verify the lodging now reflects the counterproposed values
    let resp = pub_client
        .get(&format!("{}/api/lodgings/{}", base, lodging_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let lodging: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        lodging["monthly_rent"], 2100.0,
        "lodging rent should be updated to counterproposed value"
    );
    assert_eq!(
        lodging["deposit_amount"], 2100.0,
        "lodging deposit should be updated to counterproposed value"
    );
}

// ── Authorization checks ─────────────────────────────────────────────────────

#[tokio::test]
async fn counterpropose_requires_reviewer_or_admin_role() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // Setup: Admin creates lodging + change
    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);
    let lodging_id = create_lodging(&admin_client, &admin_csrf, &base).await;
    let change_id = create_rent_change(&admin_client, &admin_csrf, &base, &lodging_id).await;

    // Publisher role must not be able to counterpropose
    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let pub_client = bearer_client(&pub_session);
    let resp = pub_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/counterpropose",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({
            "proposed_rent": 1900.0,
            "proposed_deposit": 1900.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        403,
        "Publisher must not be able to counterpropose rent changes"
    );
}

#[tokio::test]
async fn accept_counter_requires_publisher_or_admin_role() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // Setup: Admin creates lodging + change, reviewer counters
    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);
    let lodging_id = create_lodging(&admin_client, &admin_csrf, &base).await;
    let change_id = create_rent_change(&admin_client, &admin_csrf, &base, &lodging_id).await;

    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);
    let resp = rev_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/counterpropose",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({"proposed_rent": 1800.0, "proposed_deposit": 1800.0}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Reviewer must not be able to accept the counter
    let resp = rev_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/accept-counter",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &rev_csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        403,
        "Reviewer must not be able to accept their own counterproposal"
    );
}

// ── State-machine guard tests ─────────────────────────────────────────────────

#[tokio::test]
async fn counterpropose_on_already_approved_change_returns_422() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // Setup: Admin creates lodging + change and immediately approves it
    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);
    let lodging_id = create_lodging(&admin_client, &admin_csrf, &base).await;
    let change_id = create_rent_change(&admin_client, &admin_csrf, &base, &lodging_id).await;

    let resp = admin_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/approve",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &admin_csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "initial approve should succeed");

    // Now try to counterpropose an already-approved change
    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);
    let resp = rev_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/counterpropose",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({"proposed_rent": 1500.0, "proposed_deposit": 1500.0}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        422,
        "counterpropose on an already-approved change must return 422"
    );
}

#[tokio::test]
async fn accept_counter_on_pending_change_returns_422() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    // Setup: Publisher creates change but no counterproposal yet
    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let pub_client = bearer_client(&pub_session);
    let lodging_id = create_lodging(&pub_client, &pub_csrf, &base).await;
    let change_id = create_rent_change(&pub_client, &pub_csrf, &base, &lodging_id).await;

    // Try to accept-counter on a still-pending change (no counter yet)
    let resp = pub_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/accept-counter",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &pub_csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        422,
        "accept-counter on a pending (not countered) change must return 422"
    );
}

#[tokio::test]
async fn counterpropose_validates_deposit_cap() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);
    let lodging_id = create_lodging(&admin_client, &admin_csrf, &base).await;
    let change_id = create_rent_change(&admin_client, &admin_csrf, &base, &lodging_id).await;

    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);

    // Deposit 3× the rent — should violate the 1.5× cap
    let resp = rev_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/counterpropose",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({
            "proposed_rent": 1000.0,
            "proposed_deposit": 3000.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        422,
        "counterpropose with deposit exceeding 1.5x rent must return 422"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "DEPOSIT_CAP_EXCEEDED");
}

// ── Pending list reflects both statuses ──────────────────────────────────────

#[tokio::test]
async fn pending_list_includes_countered_changes() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);
    let lodging_id = create_lodging(&admin_client, &admin_csrf, &base).await;
    let change_id = create_rent_change(&admin_client, &admin_csrf, &base, &lodging_id).await;

    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);

    // Counter the change
    let resp = rev_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/counterpropose",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &rev_csrf)
        .json(&serde_json::json!({"proposed_rent": 1900.0, "proposed_deposit": 1900.0}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Pending list must contain the countered entry
    let resp = rev_client
        .get(&format!("{}/api/lodgings/rent-changes/pending", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    let entry = list
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == change_id)
        .cloned()
        .expect("countered change must appear in pending list");
    assert_eq!(entry["status"], "countered");
    assert_eq!(entry["counterproposal_rent"], 1900.0);
}
