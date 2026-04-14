use crate::helpers::*;

#[tokio::test]
async fn export_request_and_approve_flow() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    // Request export as reviewer
    let (session, csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);
    let resp = c.post(&format!("{}/api/export/request", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"export_type": "inventory"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    let export_id = body["id"].as_str().unwrap();
    assert_eq!(body["status"], "pending");

    // Download should be blocked before approval
    let resp = c.get(&format!("{}/api/export/download/{}", base_url(), export_id))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);

    // Approve as admin
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);
    let resp = c.post(&format!("{}/api/export/approve/{}", base_url(), export_id))
        .header("X-CSRF-Token", &csrf)
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "approved");
    assert!(body["watermark_text"].is_string());

    // Download should now return a valid .xlsx binary
    let resp = c.get(&format!("{}/api/export/download/{}", base_url(), export_id))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers()
        .get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("spreadsheetml"), "Expected xlsx content-type, got {}", ct);
    let cd = resp.headers()
        .get("content-disposition").unwrap().to_str().unwrap();
    assert!(cd.contains(".xlsx"), "Content-Disposition must reference .xlsx file");
    // Verify the response body is non-empty binary (PK zip magic for xlsx)
    let bytes = resp.bytes().await.unwrap();
    assert!(bytes.len() > 4, "xlsx body should not be empty");
    assert_eq!(&bytes[..4], b"PK\x03\x04", "xlsx must start with ZIP/PK magic bytes");
}

#[tokio::test]
async fn clerk_cannot_request_export() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);
    let resp = c.post(&format!("{}/api/export/request", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"export_type": "inventory"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn import_xlsx_only() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // Try uploading a non-xlsx file
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(b"not excel".to_vec())
            .file_name("data.csv")
            .mime_str("text/csv").unwrap());

    let resp = c.post(&format!("{}/api/import/upload", base_url()))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
}
