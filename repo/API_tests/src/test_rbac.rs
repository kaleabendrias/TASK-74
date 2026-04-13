use crate::helpers::*;

fn client() -> reqwest::Client { authed_client() }

#[tokio::test]
async fn clinician_cannot_create_lodging() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "clinician").await;

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({"name": "Test", "amenities": []}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn admin_can_create_resource() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    let resp = c.post(&format!("{}/api/resources", base_url()))
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
    let c = client();
    login_as(&c, "clerk").await;

    // InventoryClerk is not in the allowed roles for resources
    // The route guard should return 403
    let resp = c.post(&format!("{}/api/resources", base_url()))
        .json(&serde_json::json!({
            "title": "Test", "address": "Addr", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn reviewer_can_view_resources() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "reviewer").await;

    let resp = c.get(&format!("{}/api/resources", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn clinician_can_view_inventory() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "clinician").await;

    let resp = c.get(&format!("{}/api/inventory/lots", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn publisher_cannot_access_inventory() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "publisher").await;

    let resp = c.get(&format!("{}/api/inventory/lots", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn reviewer_cannot_create_resources() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "reviewer").await;

    // Reviewer is in the allowed_roles for update but not create (which requires Admin|Publisher)
    let resp = c.post(&format!("{}/api/resources", base_url()))
        .json(&serde_json::json!({
            "title": "Test", "address": "Addr", "tags": [], "hours": {}, "pricing": {}
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 403);
}
