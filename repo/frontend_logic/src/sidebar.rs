//! Sidebar visibility logic — which sections each role can see.
//!
//! This is the single source of truth.  Both `frontend/src/components/sidebar.rs`
//! (for conditional HTML rendering) and `frontend_tests` (for unit verification)
//! call this function.

use crate::models::UserRole;

/// Returns the sidebar sections visible for a given role, in the order they
/// appear in the sidebar.  Matches the `matches!` guards in `sidebar.rs`.
///
/// Sections: "Main", "Content", "Inventory", "Data", "System", "Account"
/// Returns the single uppercase initial letter shown in the sidebar avatar.
///
/// Mirrors the expression in `frontend/src/components/sidebar.rs`:
/// ```ignore
/// let initial = user.username.chars().next().unwrap_or('?').to_uppercase().to_string();
/// ```
pub fn avatar_initial(username: &str) -> String {
    username.chars().next().unwrap_or('?').to_uppercase().to_string()
}

pub fn visible_sections(role: &UserRole) -> Vec<&'static str> {
    let mut sections = vec!["Main", "Account"]; // always visible
    if matches!(role,
        UserRole::Administrator | UserRole::Publisher
        | UserRole::Reviewer | UserRole::Clinician)
    {
        sections.push("Content");
    }
    if matches!(role,
        UserRole::Administrator | UserRole::Clinician | UserRole::InventoryClerk)
    {
        sections.push("Inventory");
    }
    if matches!(role,
        UserRole::Administrator | UserRole::InventoryClerk | UserRole::Reviewer)
    {
        sections.push("Data");
    }
    if matches!(role, UserRole::Administrator) {
        sections.push("System");
    }
    sections
}
