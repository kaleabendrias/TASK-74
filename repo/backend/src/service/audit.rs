use diesel::PgConnection;
use uuid::Uuid;

use crate::repository::audit;

/// Records an audit log entry for a significant action.
pub fn log_action(
    conn: &mut PgConnection,
    actor_id: Uuid,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    detail: Option<serde_json::Value>,
    ip_address: Option<&str>,
) {
    let entry = audit::NewAuditEntry {
        actor_id: Some(actor_id),
        action,
        entity_type,
        entity_id,
        detail,
        ip_address,
    };
    if let Err(e) = audit::insert(conn, &entry) {
        tracing::error!(error = %e, action = action, "Failed to write audit log");
    }
}
