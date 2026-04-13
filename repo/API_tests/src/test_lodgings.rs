use crate::helpers::*;

fn client() -> reqwest::Client { authed_client() }

#[tokio::test]
async fn create_lodging_valid() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({
            "name": "Ocean View Suite",
            "description": "A lovely suite",
            "amenities": ["wifi", "pool"],
            "monthly_rent": 1000.0,
            "deposit_amount": 1500.0
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "Ocean View Suite");
    assert_eq!(body["state"], "draft");
}

#[tokio::test]
async fn deposit_cap_at_1_50x_accepted() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({
            "name": "Cap Test OK",
            "amenities": [],
            "monthly_rent": 1000.0,
            "deposit_amount": 1500.0
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
}

#[tokio::test]
async fn deposit_cap_at_1_51x_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({
            "name": "Cap Test Fail",
            "amenities": [],
            "monthly_rent": 1000.0,
            "deposit_amount": 1501.0
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "DEPOSIT_CAP_EXCEEDED");
}

#[tokio::test]
async fn vacancy_period_7_nights_ok() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    // Create lodging first
    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({"name": "Period Test", "amenities": []}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let resp = c.put(&format!("{}/api/lodgings/{}/periods", base_url(), id))
        .json(&serde_json::json!({
            "start_date": "2025-06-01",
            "end_date": "2025-06-08",
            "min_nights": 7,
            "max_nights": 365
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
}

#[tokio::test]
async fn vacancy_period_6_nights_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({"name": "Period Fail", "amenities": []}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let resp = c.put(&format!("{}/api/lodgings/{}/periods", base_url(), id))
        .json(&serde_json::json!({
            "start_date": "2025-06-01",
            "end_date": "2025-06-08",
            "min_nights": 6,
            "max_nights": 365
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn vacancy_period_366_nights_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({"name": "Long Period", "amenities": []}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let resp = c.put(&format!("{}/api/lodgings/{}/periods", base_url(), id))
        .json(&serde_json::json!({
            "start_date": "2025-06-01",
            "end_date": "2025-06-08",
            "min_nights": 7,
            "max_nights": 366
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn vacancy_period_overlap_rejected() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "admin").await;

    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({"name": "Overlap Test", "amenities": []}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    // First period
    let resp = c.put(&format!("{}/api/lodgings/{}/periods", base_url(), id))
        .json(&serde_json::json!({"start_date":"2025-06-01","end_date":"2025-06-15","min_nights":7,"max_nights":365}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);

    // Overlapping period
    let resp = c.put(&format!("{}/api/lodgings/{}/periods", base_url(), id))
        .json(&serde_json::json!({"start_date":"2025-06-10","end_date":"2025-06-25","min_nights":7,"max_nights":365}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn rent_change_approve_lifecycle() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();
    login_as(&c, "publisher").await;

    // Create lodging
    let resp = c.post(&format!("{}/api/lodgings", base_url()))
        .json(&serde_json::json!({"name":"Rent Change Test","amenities":[],"monthly_rent":1000.0,"deposit_amount":1000.0}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let lid = body["id"].as_str().unwrap();

    // Request rent change
    let resp = c.put(&format!("{}/api/lodgings/{}/rent-change", base_url(), lid))
        .json(&serde_json::json!({"proposed_rent":1200.0,"proposed_deposit":1500.0}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    let change_id = body["id"].as_str().unwrap();
    assert_eq!(body["status"], "pending");

    // Approve as reviewer
    login_as(&c, "reviewer").await;
    let resp = c.post(&format!("{}/api/lodgings/{}/rent-change/{}/approve", base_url(), lid, change_id))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "approved");

    // Verify lodging updated
    let resp = c.get(&format!("{}/api/lodgings/{}", base_url(), lid))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!((body["monthly_rent"].as_f64().unwrap() - 1200.0).abs() < 0.01);
}
