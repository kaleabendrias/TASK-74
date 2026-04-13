use crate::helpers::*;

fn client() -> reqwest::Client { authed_client() }

#[tokio::test]
async fn export_request_and_approve_flow() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();

    // Request export as clerk
    login_as(&c, "clerk").await;
    let resp = c.post(&format!("{}/api/export/request", base_url()))
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
    login_as(&c, "admin").await;
    let resp = c.post(&format!("{}/api/export/approve/{}", base_url(), export_id))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "approved");
    assert!(body["watermark_text"].is_string());

    // Download should now work
    let resp = c.get(&format!("{}/api/export/download/{}", base_url(), export_id))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["watermark"].is_string());
}

#[tokio::test]
async fn import_xlsx_only() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    // Try uploading a non-xlsx file
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(b"not excel".to_vec())
            .file_name("data.csv")
            .mime_str("text/csv").unwrap());

    let resp = c.post(&format!("{}/api/import/upload", base_url()))
        .multipart(form)
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
}
