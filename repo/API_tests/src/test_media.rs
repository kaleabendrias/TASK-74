use crate::helpers::*;

// Helper to create a minimal valid PNG (1x1 pixel)
fn minimal_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG header
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC,
        0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

#[tokio::test]
async fn upload_valid_png() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    std::fs::create_dir_all("/tmp/test_uploads").ok();

    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(minimal_png())
            .file_name("test.png")
            .mime_str("image/png").unwrap());

    let resp = c.post(&format!("{}/api/media/upload", base_url()))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["mime_type"], "image/png");
    assert!(!body["checksum_sha256"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn upload_exe_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(vec![0x4D, 0x5A, 0x00, 0x00])
            .file_name("malware.exe")
            .mime_str("application/octet-stream").unwrap());

    let resp = c.post(&format!("{}/api/media/upload", base_url()))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "INVALID_FILE_TYPE");
}

#[tokio::test]
async fn upload_jpg_extension_pdf_content_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    // PDF magic bytes with .jpg extension
    let pdf_bytes = b"%PDF-1.4 fake content".to_vec();
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(pdf_bytes)
            .file_name("sneaky.jpg")
            .mime_str("image/jpeg").unwrap());

    let resp = c.post(&format!("{}/api/media/upload", base_url()))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "MIME_MISMATCH");
}

#[tokio::test]
async fn download_uploaded_file() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    std::fs::create_dir_all("/tmp/test_uploads").ok();
    let png = minimal_png();
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(png.clone())
            .file_name("dl_test.png")
            .mime_str("image/png").unwrap());

    let resp = c.post(&format!("{}/api/media/upload", base_url()))
        .header("X-CSRF-Token", &csrf)
        .multipart(form)
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let resp = c.get(&format!("{}/api/media/{}/download", base_url(), id))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("image/png"));
}
