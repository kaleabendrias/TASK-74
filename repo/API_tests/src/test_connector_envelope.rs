//! Transport-level connector envelope tests.
//!
//! These tests exercise the inbound connector endpoint at the HTTP boundary,
//! verifying that the envelope validation (timestamp window, HMAC signature,
//! nonce deduplication) is enforced correctly regardless of the underlying
//! transport (HTTP or AMQP). The AMQP consumer re-uses the same
//! `validate_and_process` service function, so HTTP-level tests are
//! authoritative for the shared logic.
//!
//! Additional tests here focus on the atomic idempotency guarantee: two
//! concurrent requests carrying the same nonce must yield exactly one 200 and
//! one 409, never two 200s.

use crate::helpers::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const SIGNING_KEY: &str = "req-sign-key-tourism-portal-2024";

fn sign(body: &str, nonce: &str, ts: &str) -> String {
    let msg = format!("{}{}{}", body, nonce, ts);
    let mut mac = HmacSha256::new_from_slice(SIGNING_KEY.as_bytes()).unwrap();
    mac.update(msg.as_bytes());
    mac.finalize().into_bytes().iter().map(|b| format!("{:02x}", b)).collect()
}

fn connector_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

/// A well-formed connector envelope with all correct fields must be accepted.
#[tokio::test]
async fn envelope_accepted_with_valid_fields() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{"source":"envelope_test"}}"#;
    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = chrono::Utc::now().timestamp().to_string();
    let sig = sign(body, &nonce, &ts);

    let resp = connector_client()
        .post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", &nonce)
        .header("X-Timestamp", &ts)
        .header("Content-Type", "application/json")
        .body(body)
        .send().await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"], true);
    assert!(body["entity_type"].is_string());
}

/// Envelope with a timestamp exactly 4 minutes old must still be accepted
/// (within the 5-minute window).
#[tokio::test]
async fn envelope_accepted_within_time_window() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{}}"#;
    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = (chrono::Utc::now().timestamp() - 240).to_string(); // 4 min ago
    let sig = sign(body, &nonce, &ts);

    let resp = connector_client()
        .post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", &nonce)
        .header("X-Timestamp", &ts)
        .header("Content-Type", "application/json")
        .body(body)
        .send().await.unwrap();

    assert_eq!(resp.status(), 200);
}

/// Envelope with a timestamp 6 minutes old must be rejected (outside window).
#[tokio::test]
async fn envelope_rejected_outside_time_window() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{}}"#;
    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = (chrono::Utc::now().timestamp() - 360).to_string(); // 6 min ago
    let sig = sign(body, &nonce, &ts);

    let resp = connector_client()
        .post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", &nonce)
        .header("X-Timestamp", &ts)
        .header("Content-Type", "application/json")
        .body(body)
        .send().await.unwrap();

    assert_eq!(resp.status(), 401);
}

/// A tampered body (signature computed over original body) must fail HMAC
/// verification and return 401.
#[tokio::test]
async fn envelope_rejected_with_tampered_body() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let original_body = r#"{"entity_type":"resource","data":{}}"#;
    let tampered_body = r#"{"entity_type":"resource","data":{"injected":true}}"#;
    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = chrono::Utc::now().timestamp().to_string();
    // Signature is over original body but we send tampered body
    let sig = sign(original_body, &nonce, &ts);

    let resp = connector_client()
        .post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", &nonce)
        .header("X-Timestamp", &ts)
        .header("Content-Type", "application/json")
        .body(tampered_body)
        .send().await.unwrap();

    assert_eq!(resp.status(), 401);
}

/// An envelope with a missing entity_type field must return 400.
#[tokio::test]
async fn envelope_rejected_with_missing_entity_type() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"data":{"key":"value"}}"#; // entity_type absent
    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = chrono::Utc::now().timestamp().to_string();
    let sig = sign(body, &nonce, &ts);

    let resp = connector_client()
        .post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", &nonce)
        .header("X-Timestamp", &ts)
        .header("Content-Type", "application/json")
        .body(body)
        .send().await.unwrap();

    // Missing required field → 400 from JSON deserialization
    assert_eq!(resp.status(), 400);
}

/// Two sequential requests carrying the same nonce must yield the second a 409.
/// This tests the atomic idempotency guarantee; the insert uses ON CONFLICT DO
/// NOTHING so there is no TOCTOU window.
#[tokio::test]
async fn atomic_idempotency_sequential_same_nonce() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{}}"#;
    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = chrono::Utc::now().timestamp().to_string();
    let sig = sign(body, &nonce, &ts);

    let c = connector_client();

    let first = c.post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", &nonce)
        .header("X-Timestamp", &ts)
        .header("Content-Type", "application/json")
        .body(body)
        .send().await.unwrap();
    assert_eq!(first.status(), 200, "First request must be accepted");

    let second = c.post(&format!("{}/api/connector/inbound", base_url()))
        .header("Authorization", &sig)
        .header("X-Nonce", &nonce)
        .header("X-Timestamp", &ts)
        .header("Content-Type", "application/json")
        .body(body)
        .send().await.unwrap();
    assert_eq!(second.status(), 409, "Duplicate nonce must yield 409");

    let err: serde_json::Value = second.json().await.unwrap();
    assert_eq!(err["code"], "CONFLICT");
}

/// Two concurrent requests with the same nonce: exactly one must succeed (200)
/// and the other must get 409. This proves the atomicity holds under concurrent
/// load — the INSERT … ON CONFLICT pattern prevents two 200s.
#[tokio::test]
async fn atomic_idempotency_concurrent_same_nonce() {
    let pool = setup_pool();
    let _seed = seed_users(&pool);

    let body = r#"{"entity_type":"resource","data":{"concurrent":true}}"#;
    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = chrono::Utc::now().timestamp().to_string();
    let sig = sign(body, &nonce, &ts);

    let c1 = connector_client();
    let c2 = connector_client();
    let url = format!("{}/api/connector/inbound", base_url());

    let (r1, r2) = tokio::join!(
        c1.post(&url)
            .header("Authorization", &sig)
            .header("X-Nonce", &nonce)
            .header("X-Timestamp", &ts)
            .header("Content-Type", "application/json")
            .body(body)
            .send(),
        c2.post(&url)
            .header("Authorization", &sig)
            .header("X-Nonce", &nonce)
            .header("X-Timestamp", &ts)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
    );

    let s1 = r1.unwrap().status().as_u16();
    let s2 = r2.unwrap().status().as_u16();

    let accepted = [s1, s2].iter().filter(|&&s| s == 200).count();
    let rejected = [s1, s2].iter().filter(|&&s| s == 409).count();

    assert_eq!(accepted, 1, "Exactly one concurrent request must be accepted (got {}, {})", s1, s2);
    assert_eq!(rejected, 1, "Exactly one concurrent request must be rejected (got {}, {})", s1, s2);
}
