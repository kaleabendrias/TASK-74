use crate::helpers::*;

fn client() -> reqwest::Client { authed_client() }

#[tokio::test]
async fn login_success() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();

    let resp = c.post(&format!("{}/api/auth/login", base_url()))
        .json(&serde_json::json!({"username": "admin", "password": "testpassword"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["csrf_token"].is_string());
    assert!(!body["csrf_token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn login_wrong_password() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();

    let resp = c.post(&format!("{}/api/auth/login", base_url()))
        .json(&serde_json::json!({"username": "admin", "password": "wrong"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn login_nonexistent_user() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();

    let resp = c.post(&format!("{}/api/auth/login", base_url()))
        .json(&serde_json::json!({"username": "nobody", "password": "anything"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn me_returns_profile() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = authed_client();
    let (session, csrf) = login_as(&c, "admin").await;
    let c = bearer_client(&session);

    let resp = c.get(&format!("{}/api/auth/me", base_url()))
        .header("X-CSRF-Token", &csrf)
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["username"], "admin");
    assert_eq!(body["role"], "Administrator");
}

#[tokio::test]
async fn me_without_session_returns_401() {
    let c = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
    let resp = c.get(&format!("{}/api/auth/me", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn logout_clears_session() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = authed_client();
    let (session, csrf) = login_as(&c, "admin").await;
    let c = bearer_client(&session);

    let resp = c.post(&format!("{}/api/auth/logout", base_url()))
        .header("X-CSRF-Token", &csrf)
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // me should now fail
    let resp = c.get(&format!("{}/api/auth/me", base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn csrf_token_present_in_login_response() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);
    let c = client();

    let resp = c.post(&format!("{}/api/auth/login", base_url()))
        .json(&serde_json::json!({"username": "publisher", "password": "testpassword"}))
        .send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let csrf = body["csrf_token"].as_str().unwrap();
    assert!(csrf.len() > 20, "CSRF token should be substantial");
}
