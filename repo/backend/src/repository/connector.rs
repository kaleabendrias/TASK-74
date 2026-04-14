use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{idempotency_keys, api_connector_logs};

// ── Idempotency Keys ──

#[derive(Insertable)]
#[diesel(table_name = idempotency_keys)]
pub struct NewIdempotencyKey<'a> {
    pub key_value: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Uuid,
}

/// Atomically inserts a new idempotency key using `INSERT … ON CONFLICT DO NOTHING`.
///
/// Returns `Ok(1)` on success and `Ok(0)` when the nonce already existed, so
/// callers can map a `0` result to a 409 Conflict without an extra SELECT
/// round-trip. This eliminates the TOCTOU race condition that existed when a
/// separate `nonce_exists()` check preceded the insert.
pub fn insert_idempotency_key_atomic(
    conn: &mut PgConnection,
    new: &NewIdempotencyKey,
) -> QueryResult<usize> {
    diesel::insert_into(idempotency_keys::table)
        .values(new)
        .on_conflict(idempotency_keys::key_value)
        .do_nothing()
        .execute(conn)
}

// ── Connector Logs ──

#[derive(Insertable)]
#[diesel(table_name = api_connector_logs)]
pub struct NewConnectorLog<'a> {
    pub direction: &'a str,
    pub endpoint: &'a str,
    pub nonce: Option<&'a str>,
    pub timestamp_sent: Option<DateTime<Utc>>,
    pub payload_hash: Option<&'a str>,
    pub status: &'a str,
}

/// Inserts a log entry for an API connector request or response.
pub fn insert_connector_log(
    conn: &mut PgConnection,
    new: &NewConnectorLog,
) -> QueryResult<usize> {
    diesel::insert_into(api_connector_logs::table)
        .values(new)
        .execute(conn)
}
