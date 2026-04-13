use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::audit_log;

#[derive(Insertable)]
#[diesel(table_name = audit_log)]
pub struct NewAuditEntry<'a> {
    pub actor_id: Option<Uuid>,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Option<Uuid>,
    pub detail: Option<serde_json::Value>,
    pub ip_address: Option<&'a str>,
}

/// Inserts a new entry into the audit log.
pub fn insert(conn: &mut PgConnection, entry: &NewAuditEntry) -> QueryResult<usize> {
    diesel::insert_into(audit_log::table)
        .values(entry)
        .execute(conn)
}
