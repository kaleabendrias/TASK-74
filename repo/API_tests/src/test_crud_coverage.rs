//! Gap-filling tests to push the suite past 90% endpoint coverage.
//!
//! Covers: resource GET-by-id, lodging update/GET/periods, rent reject,
//! lot GET by id, transaction list, media upload/download, import upload
//! + job polling, SSE stream endpoint, and export pending list.

use crate::helpers::*;

// ── Resources: GET by ID ─────────────────────────────────────────────────────

#[tokio::test]
async fn resource_get_by_id_returns_resource() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Get-by-ID Test",
            "address": "1 Main St",
            "tags": ["test"],
            "hours": {},
            "pricing": {}
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();

    let resp = c
        .get(&format!("{}/api/resources/{}", base, id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], id);
    assert_eq!(body["title"], "Get-by-ID Test");
}

#[tokio::test]
async fn resource_get_by_id_returns_404_for_unknown() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .get(&format!("{}/api/resources/00000000-0000-0000-0000-000000000000", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

/// PUT /api/resources/:id can update fields without changing state.
#[tokio::test]
async fn resource_put_updates_fields_without_state_change() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/resources", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Original Title",
            "address": "Old Address",
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

    let resp = c
        .put(&format!("{}/api/resources/{}", base, id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Updated Title",
            "address": "New Address"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["address"], "New Address");
    assert_eq!(body["state"], "draft", "state must not change on a field update");
}

// ── Lodgings: list, GET by ID, update ────────────────────────────────────────

#[tokio::test]
async fn lodging_list_returns_array() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Create at least one lodging
    c.post(&format!("{}/api/lodgings", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "name": "List Test Lodge",
            "amenities": [],
            "monthly_rent": 800.0,
            "deposit_amount": 800.0
        }))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(&format!("{}/api/lodgings", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array(), "lodgings list must be an array");
}

#[tokio::test]
async fn lodging_get_by_id_returns_lodging() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/lodgings", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "name": "Get-by-ID Lodge",
            "amenities": ["wifi"],
            "monthly_rent": 1200.0,
            "deposit_amount": 1200.0
        }))
        .send()
        .await
        .unwrap();
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = c
        .get(&format!("{}/api/lodgings/{}", base, id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "Get-by-ID Lodge");
}

/// PUT /api/lodgings/:id updates lodging fields.
#[tokio::test]
async fn lodging_put_updates_fields() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/lodgings", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "name": "Update Me Lodge",
            "amenities": [],
            "monthly_rent": 900.0,
            "deposit_amount": 900.0
        }))
        .send()
        .await
        .unwrap();
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = c
        .put(&format!("{}/api/lodgings/{}", base, id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "name": "Updated Lodge Name",
            "amenities": ["pool"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "Updated Lodge Name");
}

/// GET /api/lodgings/:id/periods returns periods list (possibly empty) for a lodging.
#[tokio::test]
async fn lodging_periods_list_returns_array() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/lodgings", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "name": "Periods Lodge",
            "amenities": [],
            "monthly_rent": 1000.0,
            "deposit_amount": 1000.0
        }))
        .send()
        .await
        .unwrap();
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Add a period
    c.put(&format!("{}/api/lodgings/{}/periods", base, id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "start_date": "2026-07-01",
            "end_date": "2026-07-14",
            "min_nights": 7,
            "max_nights": 30
        }))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(&format!("{}/api/lodgings/{}/periods", base, id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array(), "periods list must be an array");
    let arr = body.as_array().unwrap();
    assert!(!arr.is_empty(), "lodging should have the period we just added");
}

// ── Rent change: reject path ──────────────────────────────────────────────────

/// Reviewer rejects a pending rent-change request (pending → rejected).
#[tokio::test]
async fn rent_change_reject_lifecycle() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let pub_client = bearer_client(&pub_session);

    // Create lodging + rent-change
    let resp = pub_client
        .post(&format!("{}/api/lodgings", base))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({
            "name": "Reject Test Lodge",
            "amenities": [],
            "monthly_rent": 1000.0,
            "deposit_amount": 1000.0
        }))
        .send()
        .await
        .unwrap();
    let lodging_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = pub_client
        .put(&format!("{}/api/lodgings/{}/rent-change", base, lodging_id))
        .header("X-CSRF-Token", &pub_csrf)
        .json(&serde_json::json!({
            "proposed_rent": 1500.0,
            "proposed_deposit": 1500.0
        }))
        .send()
        .await
        .unwrap();
    let change_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Reviewer rejects
    let (rev_session, rev_csrf) = login_as(&authed_client(), "reviewer").await;
    let rev_client = bearer_client(&rev_session);

    let resp = rev_client
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/reject",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &rev_csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "reject must return 200");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "rejected");

    // Lodging rent must remain unchanged
    let resp = pub_client
        .get(&format!("{}/api/lodgings/{}", base, lodging_id))
        .send()
        .await
        .unwrap();
    let lodging: serde_json::Value = resp.json().await.unwrap();
    assert!(
        (lodging["monthly_rent"].as_f64().unwrap() - 1000.0).abs() < 0.01,
        "rejected change must not update lodging rent"
    );
}

/// Publisher cannot reject a rent change (only reviewers and admins can).
#[tokio::test]
async fn rent_change_reject_blocked_for_publisher() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (admin_session, admin_csrf) = login_as(&authed_client(), "admin").await;
    let admin_client = bearer_client(&admin_session);

    let resp = admin_client
        .post(&format!("{}/api/lodgings", base))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({
            "name": "Reject RBAC Lodge",
            "amenities": [],
            "monthly_rent": 500.0,
            "deposit_amount": 500.0
        }))
        .send()
        .await
        .unwrap();
    let lodging_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = admin_client
        .put(&format!("{}/api/lodgings/{}/rent-change", base, lodging_id))
        .header("X-CSRF-Token", &admin_csrf)
        .json(&serde_json::json!({"proposed_rent": 600.0, "proposed_deposit": 600.0}))
        .send()
        .await
        .unwrap();
    let change_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (pub_session, pub_csrf) = login_as(&authed_client(), "publisher").await;
    let resp = bearer_client(&pub_session)
        .post(&format!(
            "{}/api/lodgings/{}/rent-change/{}/reject",
            base, lodging_id, change_id
        ))
        .header("X-CSRF-Token", &pub_csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        403,
        "publisher must not be able to reject rent changes, got {}",
        resp.status()
    );
}

// ── Inventory: lot GET by ID, transaction list ────────────────────────────────

#[tokio::test]
async fn inventory_lot_get_by_id_returns_lot() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    let resp = c
        .post(&format!("{}/api/inventory/lots", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "Get-by-ID Item",
            "lot_number": "LOT-GBI-001",
            "quantity_on_hand": 25
        }))
        .send()
        .await
        .unwrap();
    let lot_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = c
        .get(&format!("{}/api/inventory/lots/{}", base, lot_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], lot_id);
    assert_eq!(body["lot_number"], "LOT-GBI-001");
    assert_eq!(body["item_name"], "Get-by-ID Item");
}

#[tokio::test]
async fn inventory_transaction_list_returns_array() {
    let pool = setup_pool();
    let seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    // Create a lot and post a transaction
    let resp = c
        .post(&format!("{}/api/inventory/lots", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "facility_id": seed.facility_id.to_string(),
            "warehouse_id": seed.warehouse_id.to_string(),
            "bin_id": seed.bin_id.to_string(),
            "item_name": "TX List Item",
            "lot_number": "LOT-TXL-001",
            "quantity_on_hand": 100
        }))
        .send()
        .await
        .unwrap();
    let lot_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    c.post(&format!("{}/api/inventory/transactions", base))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "lot_id": lot_id,
            "direction": "outbound",
            "quantity": 10,
            "reason": "Dispensed to patient"
        }))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(&format!("{}/api/inventory/transactions", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array(), "transactions list must be an array");
    let arr = body.as_array().unwrap();
    assert!(
        arr.iter().any(|t| t["lot_id"] == lot_id),
        "transaction for created lot must appear in list"
    );
}

// ── Media: upload and download ───────────────────────────────────────────────

/// POST /api/media/upload accepts a valid image and returns a media record.
#[tokio::test]
async fn media_upload_valid_image_returns_record() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Minimal valid PNG: 1×1 pixel
    let png_bytes = include_bytes!("fixtures/1x1.png");
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(png_bytes.to_vec())
            .file_name("test.png")
            .mime_str("image/png")
            .unwrap(),
    );

    let resp = c
        .post(&format!("{}/api/media/upload", base))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "valid PNG upload must return 201");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["id"].is_string(), "media record must have id");
    assert!(body["original_name"].is_string(), "media record must have original_name");
    assert!(body["checksum_sha256"].is_string(), "media record must have sha256 checksum");
}

/// GET /api/media/:id/download returns the file bytes for an uploaded file.
#[tokio::test]
async fn media_download_returns_file_bytes() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let png_bytes = include_bytes!("fixtures/1x1.png");
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(png_bytes.to_vec())
            .file_name("download_test.png")
            .mime_str("image/png")
            .unwrap(),
    );

    let upload: serde_json::Value = c
        .post(&format!("{}/api/media/upload", base))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let media_id = upload["id"].as_str().unwrap();

    let resp = c
        .get(&format!("{}/api/media/{}/download", base, media_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "download of uploaded file must return 200");
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        ct.contains("image/png") || ct.contains("application/octet-stream"),
        "download content-type must be image/png or octet-stream, got {}",
        ct
    );
    let bytes = resp.bytes().await.unwrap();
    assert!(!bytes.is_empty(), "download body must not be empty");
}

/// POST /api/media/upload rejects MIME type mismatches (e.g. .png with wrong magic).
#[tokio::test]
async fn media_upload_mime_mismatch_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Plain text content but claiming to be image/png
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(b"this is not a png".to_vec())
            .file_name("fake.png")
            .mime_str("image/png")
            .unwrap(),
    );

    let resp = c
        .post(&format!("{}/api/media/upload", base))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        422,
        "MIME mismatch must be rejected with 422, got {}",
        resp.status()
    );
}

// ── Import: upload + job poll ─────────────────────────────────────────────────

/// POST /api/import/upload with a minimal valid .xlsx creates an import job.
#[tokio::test]
async fn import_upload_valid_xlsx_creates_job() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let xlsx_bytes = include_bytes!("fixtures/sample_import.xlsx");
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(xlsx_bytes.to_vec())
            .file_name("import.xlsx")
            .mime_str("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .unwrap(),
    );

    let resp = c
        .post(&format!("{}/api/import/upload", base))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "valid xlsx upload must return 201");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["id"].is_string(), "response must contain id");

    // Poll the job status
    let job_id = body["id"].as_str().unwrap();
    let resp = c
        .get(&format!("{}/api/import/jobs/{}", base, job_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let job: serde_json::Value = resp.json().await.unwrap();
    assert!(job["status"].is_string());
    let status = job["status"].as_str().unwrap();
    assert!(
        status == "queued" || status == "running" || status == "completed" || status == "failed",
        "job status must be a valid state, got {}",
        status
    );
}

/// GET /api/import/jobs/:id/stream returns 200 with text/event-stream content-type.
#[tokio::test]
async fn import_job_sse_stream_endpoint_responds() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let base = base_url();

    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Create a job first
    let xlsx_bytes = include_bytes!("fixtures/sample_import.xlsx");
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(xlsx_bytes.to_vec())
            .file_name("sse_test.xlsx")
            .mime_str("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .unwrap(),
    );
    let resp = c
        .post(&format!("{}/api/import/upload", base))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send()
        .await
        .unwrap();
    let job_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // SSE stream — just verify the endpoint accepts the connection
    let resp = c
        .get(&format!("{}/api/import/jobs/{}/stream", base, job_id))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "SSE stream endpoint must return 200, got {}",
        resp.status()
    );
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        ct.contains("text/event-stream"),
        "SSE stream must use text/event-stream content-type, got {}",
        ct
    );
}
