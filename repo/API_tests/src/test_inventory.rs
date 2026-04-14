use crate::helpers::*;

// ── Cross-facility isolation tests ──────────────────────────────────────────

/// A clerk scoped to facility 1 must not list bins from a warehouse that
/// belongs to facility 2 — the backend must return 403 or 404.
#[tokio::test]
async fn clerk_cannot_list_bins_from_other_facility() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);
    let _ = csrf; // CSRF not needed for GET

    // Clerk belongs to facility_id, warehouse2_id belongs to facility2_id.
    let resp = c.get(&format!(
        "{}/api/inventory/bins?warehouse_id={}",
        base_url(), seed.warehouse2_id
    ))
    .send().await.unwrap();
    assert!(
        resp.status() == 403 || resp.status() == 404,
        "expected 403 or 404 but got {}",
        resp.status()
    );
}

/// create_lot must reject a request where warehouse_id belongs to a different
/// facility than the declared facility_id — the backend returns 422.
#[tokio::test]
async fn create_lot_rejects_warehouse_from_wrong_facility() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    // Use admin so we are not additionally blocked by facility-scope guards.
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // facility_id = facility 1, but warehouse_id belongs to facility 2.
    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse2_id.to_string(),
            "bin_id": seed.bin2_id.to_string(),
            "item_name": "Invalid",
            "lot_number": "LOT-XFAC",
            "quantity_on_hand": 1
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "INVALID_LOCATION");
}

/// create_lot must reject a request where bin_id belongs to a different
/// warehouse than the declared warehouse_id — the backend returns 422.
#[tokio::test]
async fn create_lot_rejects_bin_from_wrong_warehouse() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // facility_id and warehouse_id are consistent (both facility 1), but
    // bin2_id belongs to warehouse2 (facility 2).
    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin2_id.to_string(),
            "item_name": "Invalid Bin",
            "lot_number": "LOT-XBIN",
            "quantity_on_hand": 1
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "INVALID_LOCATION");
}

#[tokio::test]
async fn create_lot_and_list() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Bandages",
            "lot_number": "LOT-001",
            "quantity_on_hand": 100
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);

    let resp = c.get(&format!("{}/api/inventory/lots", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn reserve_success() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Gauze",
            "lot_number": "LOT-002",
            "quantity_on_hand": 50
        }))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let lot_id = body["id"].as_str().unwrap();

    let resp = c.post(&format!("{}/api/inventory/lots/{}/reserve", base_url(), lot_id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"quantity": 10}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["quantity_on_hand"], 40);
    assert_eq!(body["quantity_reserved"], 10);
}

#[tokio::test]
async fn over_reservation_returns_409() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Syringes",
            "lot_number": "LOT-003",
            "quantity_on_hand": 5
        }))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let lot_id = body["id"].as_str().unwrap();

    let resp = c.post(&format!("{}/api/inventory/lots/{}/reserve", base_url(), lot_id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"quantity": 10}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn transaction_recorded() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Gloves",
            "lot_number": "LOT-004",
            "quantity_on_hand": 200
        }))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let lot_id = body["id"].as_str().unwrap();

    let resp = c.post(&format!("{}/api/inventory/transactions", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "lot_id": lot_id,
            "direction": "inbound",
            "quantity": 50,
            "reason": "Restocking"
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["direction"], "inbound");
    assert_eq!(body["is_immutable"], true);
}

#[tokio::test]
async fn audit_print_returns_html() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Masks",
            "lot_number": "LOT-005",
            "quantity_on_hand": 100
        }))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let lot_id = body["id"].as_str().unwrap();

    let resp = c.get(&format!("{}/api/inventory/transactions/audit-print?lot_id={}", base_url(), lot_id))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("text/html"));
    let html = resp.text().await.unwrap();
    assert!(html.contains("Audit Trail"));
}

#[tokio::test]
async fn near_expiry_filter() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    // Create a lot expiring in 10 days
    let expires = (chrono::Utc::now() + chrono::Duration::days(10)).format("%Y-%m-%d").to_string();
    let resp = c.post(&format!("{}/api/inventory/lots", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Expiring Soon",
            "lot_number": "LOT-EXP",
            "quantity_on_hand": 10,
            "expiration_date": expires
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);

    let resp = c.get(&format!("{}/api/inventory/lots?near_expiry=true", base_url()))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let items = body.as_array().unwrap();
    assert!(items.iter().any(|l| l["lot_number"] == "LOT-EXP"));
}
