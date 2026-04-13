use crate::helpers::*;

#[tokio::test]
async fn create_resource_valid() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Beach Resort",
            "category": "hotel",
            "address": "456 Ocean Dr, Miami, FL 33139",
            "tags": ["beach", "resort"],
            "hours": {"monday": {"open": "09:00", "close": "21:00"}},
            "pricing": {"adult": 299.99, "child": 149.99},
            "latitude": 25.7617,
            "longitude": -80.1918
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["title"], "Beach Resort");
    assert_eq!(body["state"], "draft");
    assert_eq!(body["current_version"], 1);
}

#[tokio::test]
async fn create_resource_missing_title_fails() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "",
            "address": "123 Main",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn create_resource_negative_pricing_fails() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Test",
            "address": "123 Main",
            "tags": [],
            "hours": {},
            "pricing": {"adult": -10.0}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn create_resource_invalid_coords_fails() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Test",
            "address": "123 Main",
            "tags": [],
            "hours": {},
            "pricing": {},
            "latitude": 999.0,
            "longitude": -999.0
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn create_resource_too_many_tags_fails() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let tags: Vec<String> = (0..21).map(|i| format!("tag{}", i)).collect();
    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Test",
            "address": "123 Main",
            "tags": tags,
            "hours": {},
            "pricing": {}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn state_transition_full_lifecycle() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    // Create as publisher
    let (session, csrf) = login_as(&authed_client(), "publisher").await;
    let c = bearer_client(&session);
    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Lifecycle Test",
            "address": "123 Main",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap();
    assert_eq!(body["state"], "draft");

    // Submit for review (publisher)
    let resp = c.put(&format!("{}/api/resources/{}", base_url(), id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"state": "in_review"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["state"], "in_review");

    // Publish (need reviewer)
    let (session, csrf) = login_as(&authed_client(), "reviewer").await;
    let c = bearer_client(&session);
    let resp = c.put(&format!("{}/api/resources/{}", base_url(), id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"state": "published"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["state"], "published");

    // Take offline (publisher)
    let (session, csrf) = login_as(&authed_client(), "publisher").await;
    let c = bearer_client(&session);
    let resp = c.put(&format!("{}/api/resources/{}", base_url(), id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"state": "offline"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Return to draft
    let resp = c.put(&format!("{}/api/resources/{}", base_url(), id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"state": "draft"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["state"], "draft");
}

#[tokio::test]
async fn version_increments_on_edit() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Version Test",
            "address": "123 Main",
            "tags": [],
            "hours": {},
            "pricing": {}
        }))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap();
    assert_eq!(body["current_version"], 1);

    // Edit
    let resp = c.put(&format!("{}/api/resources/{}", base_url(), id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"title": "Updated Title"}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["current_version"], 2);

    // Edit again
    let resp = c.put(&format!("{}/api/resources/{}", base_url(), id))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({"title": "Updated Again"}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["current_version"], 3);
}

#[tokio::test]
async fn list_resources_paginated() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, _csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.get(&format!("{}/api/resources?page=1&per_page=5", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["data"].is_array());
    assert!(body["page"].as_i64().unwrap() >= 1);
    assert!(body["per_page"].as_i64().unwrap() <= 100);
}

#[tokio::test]
async fn scheduled_publish_stored() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let (session, csrf) = login_as(&authed_client(), "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/resources", base_url()))
        .header("X-CSRF-Token", &csrf)
        .json(&serde_json::json!({
            "title": "Scheduled",
            "address": "123 Main",
            "tags": [],
            "hours": {},
            "pricing": {},
            "scheduled_publish_at": "2025-12-25T10:00:00"
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["scheduled_publish_at"].is_string());
}
