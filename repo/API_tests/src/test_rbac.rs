use crate::helpers::*;

#[tokio::test]
async fn clinician_cannot_create_lodging() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clinician").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"name": "Test", "amenities": []}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
    // Response body must conform to ApiErrorBody schema
    let body: serde_json::Value = resp.json().await.expect("403 body must be JSON");
    assert_eq!(body["code"].as_str().unwrap_or(""), "FORBIDDEN", "code field mismatch: {body}");
    assert!(body["message"].as_str().is_some(), "message must be a string: {body}");
}

#[tokio::test]
async fn admin_can_create_resource() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Test Resource",
            "address": "123 Main St",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send().await.unwrap();
    assert!(resp.status() == 201 || resp.status() == 200);
}

#[tokio::test]
async fn inventory_clerk_cannot_access_resources() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "clerk").await;
    let c = bearer_client(&session);

    // InventoryClerk is not in the allowed roles for resources
    // The route guard should return 403
    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Test", "address": "Addr", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.expect("403 body must be JSON");
    assert_eq!(body["code"].as_str().unwrap_or(""), "FORBIDDEN", "code field mismatch: {body}");
    assert!(body["message"].as_str().is_some(), "message must be a string: {body}");
}

#[tokio::test]
async fn reviewer_can_view_resources() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);

    let resp = c.get(&format!("{}/api/resources", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn clinician_can_view_inventory() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "clinician").await;
    let c = bearer_client(&session);

    let resp = c.get(&format!("{}/api/inventory/lots", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn publisher_cannot_access_inventory() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "publisher").await;
    let c = bearer_client(&session);

    let resp = c.get(&format!("{}/api/inventory/lots", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.expect("403 body must be JSON");
    assert_eq!(body["code"].as_str().unwrap_or(""), "FORBIDDEN", "code field mismatch: {body}");
    assert!(body["message"].as_str().is_some(), "message must be a string: {body}");
}

#[tokio::test]
async fn reviewer_cannot_create_resources() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);

    // Reviewer is in the allowed_roles for update but not create (which requires Admin|Publisher)
    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Test", "address": "Addr", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.expect("403 body must be JSON");
    assert_eq!(body["code"].as_str().unwrap_or(""), "FORBIDDEN", "code field mismatch: {body}");
    assert!(body["message"].as_str().is_some(), "message must be a string: {body}");
}
