use crate::helpers::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn sign_request(key: &str, body: &str, nonce: &str, timestamp: &str) -> String {
    let message = format!("{}{}{}", body, nonce, timestamp);
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    mac.finalize().into_bytes().iter().map(|b| format!("{:02x}", b)).collect()
}

fn client() -> reqwest::Client { authed_client() }

#[tokio::test]
async fn connector_valid_payload() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{"key":"value"}}"#;
    let nonce = &uuid::Uuid::new_v4().to_string();
    let ts = &chrono::Utc::now().timestamp().to_string();
    let sig = sign_request("test-signing-key", body, nonce, ts);

    let c = reqwest::Client::new();
    let resp = c.post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", nonce)
        .header("X-Timestamp", ts)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let b: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(b["accepted"], true);
}

#[tokio::test]
async fn connector_expired_timestamp() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{}}"#;
    let nonce = &uuid::Uuid::new_v4().to_string();
    let old_ts = &(chrono::Utc::now().timestamp() - 600).to_string(); // 10 min ago
    let sig = sign_request("test-signing-key", body, nonce, old_ts);

    let c = reqwest::Client::new();
    let resp = c.post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", nonce)
        .header("X-Timestamp", old_ts)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send().await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn connector_replayed_nonce() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{}}"#;
    let nonce = &uuid::Uuid::new_v4().to_string();
    let ts = &chrono::Utc::now().timestamp().to_string();
    let sig = sign_request("test-signing-key", body, nonce, ts);

    let c = reqwest::Client::new();
    // First request succeeds
    let resp = c.post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", nonce)
        .header("X-Timestamp", ts)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Replay with same nonce
    let resp = c.post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", nonce)
        .header("X-Timestamp", ts)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send().await.unwrap();
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn connector_invalid_signature() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{}}"#;
    let nonce = &uuid::Uuid::new_v4().to_string();
    let ts = &chrono::Utc::now().timestamp().to_string();

    let c = reqwest::Client::new();
    let resp = c.post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", "bad_signature")
        .header("X-Nonce", nonce)
        .header("X-Timestamp", ts)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send().await.unwrap();
    assert_eq!(resp.status(), 401);
}
