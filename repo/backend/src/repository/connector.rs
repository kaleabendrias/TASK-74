use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{idempotency_keys, api_connector_logs};

// ── Idempotency Keys ──

pub fn nonce_exists(conn: &mut PgConnection, nonce: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    diesel::select(exists(
        idempotency_keys::table.filter(idempotency_keys::key_value.eq(nonce)),
    ))
    .get_result(conn)
}

#[derive(Insertable)]
#[diesel(table_name = idempotency_keys)]
pub struct NewIdempotencyKey<'a> {
    pub key_value: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Uuid,
}

pub fn insert_idempotency_key(
    conn: &mut PgConnection,
    new: &NewIdempotencyKey,
) -> QueryResult<usize> {
    diesel::insert_into(idempotency_keys::table)
        .values(new)
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

pub fn insert_connector_log(
    conn: &mut PgConnection,
    new: &NewConnectorLog,
) -> QueryResult<usize> {
    diesel::insert_into(api_connector_logs::table)
        .values(new)
        .execute(conn)
}
