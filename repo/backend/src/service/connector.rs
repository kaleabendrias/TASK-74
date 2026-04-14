use chrono::Utc;
use diesel::PgConnection;
use uuid::Uuid;

use crate::crypto::hmac_sign;
use crate::errors::ApiError;
use crate::model::{ConnectorAck, ConnectorPayload};
use crate::repository::connector as repo;

/// Validates an inbound connector request (timestamp, nonce, HMAC signature) and logs it.
pub fn validate_and_process(
    conn: &mut PgConnection,
    signing_key: &str,
    auth_header: &str,
    body: &[u8],
    nonce: &str,
    timestamp: &str,
    endpoint: &str,
) -> Result<ConnectorAck, ApiError> {
    // Parse timestamp
    let ts: i64 = timestamp
        .parse()
        .map_err(|_| ApiError::bad_request("INVALID_TIMESTAMP", "Timestamp must be a Unix epoch"))?;

    let sent_time = chrono::DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| ApiError::bad_request("INVALID_TIMESTAMP", "Invalid Unix timestamp"))?;

    // Reject replay: >5 minutes
    let now = Utc::now();
    if (now - sent_time).num_seconds().abs() > 300 {
        return Err(ApiError::unauthorized("Request timestamp outside allowed window (5 minutes)"));
    }

    // Verify HMAC signature: sign(body + nonce + timestamp)
    // Done before the idempotency insert so invalid signatures never pollute
    // the idempotency table.
    let body_str = String::from_utf8_lossy(body);
    let message = format!("{}{}{}", body_str, nonce, timestamp);
    if !hmac_sign::verify_signature(signing_key, &message, auth_header) {
        return Err(ApiError::unauthorized("Invalid HMAC signature"));
    }

    // Parse payload
    let payload: ConnectorPayload = serde_json::from_slice(body)
        .map_err(|e| ApiError::bad_request("INVALID_PAYLOAD", &e.to_string()))?;

    let entity_id = payload.entity_id.unwrap_or_else(Uuid::new_v4);

    // Atomic idempotency insert — INSERT … ON CONFLICT DO NOTHING.
    // Returns 0 rows affected when the nonce already exists, mapping to 409
    // without a prior SELECT (eliminates the TOCTOU window).
    let inserted = repo::insert_idempotency_key_atomic(
        conn,
        &repo::NewIdempotencyKey {
            key_value: nonce,
            entity_type: &payload.entity_type,
            entity_id,
        },
    )?;
    if inserted == 0 {
        return Err(ApiError::conflict("Duplicate nonce — request already processed"));
    }

    // Log the connector call
    let payload_hash = crate::crypto::sha256::hash_bytes(body);
    repo::insert_connector_log(
        conn,
        &repo::NewConnectorLog {
            direction: "inbound",
            endpoint,
            nonce: Some(nonce),
            timestamp_sent: Some(sent_time),
            payload_hash: Some(&payload_hash),
            status: "accepted",
        },
    )?;

    Ok(ConnectorAck {
        accepted: true,
        entity_type: payload.entity_type,
        entity_id: Some(entity_id),
    })
}
