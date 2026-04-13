// RBAC permission definitions.
// The actual enforcement is done via the RbacContext extractor and the
// require_role!() macro. This module defines the permission matrix for
// documentation and potential programmatic checks.

use crate::model::UserRole;

/// Returns whether the given role is permitted to perform the specified action.
pub fn has_permission(role: UserRole, action: &str) -> bool {
    match role {
        UserRole::Administrator => true,
        UserRole::Publisher => matches!(
            action,
            "resource:create"
                | "resource:edit"
                | "resource:publish"
                | "resource:submit_review"
                | "lodging:create"
                | "lodging:edit"
                | "lodging:submit_review"
                | "media:upload"
        ),
        UserRole::Reviewer => matches!(
            action,
            "resource:review"
                | "resource:view"
                | "lodging:review"
                | "lodging:view"
                | "export:request"
                | "rent_change:approve"
                | "rent_change:reject"
        ),
        UserRole::Clinician => matches!(
            action,
            "resource:view" | "lodging:view" | "inventory:view"
        ),
        UserRole::InventoryClerk => matches!(
            action,
            "inventory:view"
                | "inventory:create"
                | "inventory:transact"
                | "inventory:import"
                | "inventory:reserve"
        ),
    }
}
