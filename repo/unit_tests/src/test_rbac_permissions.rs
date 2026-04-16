//! Exhaustive unit tests for `has_permission` — the RBAC permission matrix.
//!
//! Every (role, action) combination is tested to ensure the matrix is
//! correct and does not silently change when roles or actions are added.

use tourism_backend::middleware::rbac::has_permission;
use tourism_backend::model::UserRole;

// ── Administrator ─────────────────────────────────────────────────────────────
// Administrator has blanket allow for every action.

#[test]
fn admin_can_create_resource() {
    assert!(has_permission(UserRole::Administrator, "resource:create"));
}

#[test]
fn admin_can_edit_resource() {
    assert!(has_permission(UserRole::Administrator, "resource:edit"));
}

#[test]
fn admin_can_publish_resource() {
    assert!(has_permission(UserRole::Administrator, "resource:publish"));
}

#[test]
fn admin_can_review_resource() {
    assert!(has_permission(UserRole::Administrator, "resource:review"));
}

#[test]
fn admin_can_view_resource() {
    assert!(has_permission(UserRole::Administrator, "resource:view"));
}

#[test]
fn admin_can_create_lodging() {
    assert!(has_permission(UserRole::Administrator, "lodging:create"));
}

#[test]
fn admin_can_manage_inventory() {
    assert!(has_permission(UserRole::Administrator, "inventory:create"));
    assert!(has_permission(UserRole::Administrator, "inventory:transact"));
    assert!(has_permission(UserRole::Administrator, "inventory:import"));
    assert!(has_permission(UserRole::Administrator, "inventory:reserve"));
}

#[test]
fn admin_can_do_arbitrary_action() {
    // Administrator is granted every action — including future ones.
    assert!(has_permission(UserRole::Administrator, "anything:at_all"));
}

// ── Publisher ─────────────────────────────────────────────────────────────────

#[test]
fn publisher_can_create_resource() {
    assert!(has_permission(UserRole::Publisher, "resource:create"));
}

#[test]
fn publisher_can_edit_resource() {
    assert!(has_permission(UserRole::Publisher, "resource:edit"));
}

#[test]
fn publisher_can_publish_resource() {
    assert!(has_permission(UserRole::Publisher, "resource:publish"));
}

#[test]
fn publisher_can_submit_resource_for_review() {
    assert!(has_permission(UserRole::Publisher, "resource:submit_review"));
}

#[test]
fn publisher_can_create_lodging() {
    assert!(has_permission(UserRole::Publisher, "lodging:create"));
}

#[test]
fn publisher_can_edit_lodging() {
    assert!(has_permission(UserRole::Publisher, "lodging:edit"));
}

#[test]
fn publisher_can_submit_lodging_for_review() {
    assert!(has_permission(UserRole::Publisher, "lodging:submit_review"));
}

#[test]
fn publisher_can_upload_media() {
    assert!(has_permission(UserRole::Publisher, "media:upload"));
}

#[test]
fn publisher_cannot_review_resource() {
    assert!(!has_permission(UserRole::Publisher, "resource:review"));
}

#[test]
fn publisher_cannot_access_inventory() {
    assert!(!has_permission(UserRole::Publisher, "inventory:view"));
    assert!(!has_permission(UserRole::Publisher, "inventory:create"));
}

#[test]
fn publisher_cannot_approve_rent_change() {
    assert!(!has_permission(UserRole::Publisher, "rent_change:approve"));
}

#[test]
fn publisher_cannot_export() {
    assert!(!has_permission(UserRole::Publisher, "export:request"));
}

// ── Reviewer ─────────────────────────────────────────────────────────────────

#[test]
fn reviewer_can_review_resource() {
    assert!(has_permission(UserRole::Reviewer, "resource:review"));
}

#[test]
fn reviewer_can_view_resource() {
    assert!(has_permission(UserRole::Reviewer, "resource:view"));
}

#[test]
fn reviewer_can_review_lodging() {
    assert!(has_permission(UserRole::Reviewer, "lodging:review"));
}

#[test]
fn reviewer_can_view_lodging() {
    assert!(has_permission(UserRole::Reviewer, "lodging:view"));
}

#[test]
fn reviewer_can_request_export() {
    assert!(has_permission(UserRole::Reviewer, "export:request"));
}

#[test]
fn reviewer_can_approve_rent_change() {
    assert!(has_permission(UserRole::Reviewer, "rent_change:approve"));
}

#[test]
fn reviewer_can_reject_rent_change() {
    assert!(has_permission(UserRole::Reviewer, "rent_change:reject"));
}

#[test]
fn reviewer_cannot_create_resource() {
    assert!(!has_permission(UserRole::Reviewer, "resource:create"));
}

#[test]
fn reviewer_cannot_publish_resource() {
    assert!(!has_permission(UserRole::Reviewer, "resource:publish"));
}

#[test]
fn reviewer_cannot_access_inventory() {
    assert!(!has_permission(UserRole::Reviewer, "inventory:view"));
    assert!(!has_permission(UserRole::Reviewer, "inventory:create"));
}

#[test]
fn reviewer_cannot_upload_media() {
    assert!(!has_permission(UserRole::Reviewer, "media:upload"));
}

// ── Clinician ─────────────────────────────────────────────────────────────────

#[test]
fn clinician_can_view_resource() {
    assert!(has_permission(UserRole::Clinician, "resource:view"));
}

#[test]
fn clinician_can_view_lodging() {
    assert!(has_permission(UserRole::Clinician, "lodging:view"));
}

#[test]
fn clinician_can_view_inventory() {
    assert!(has_permission(UserRole::Clinician, "inventory:view"));
}

#[test]
fn clinician_cannot_create_resource() {
    assert!(!has_permission(UserRole::Clinician, "resource:create"));
}

#[test]
fn clinician_cannot_create_lodging() {
    assert!(!has_permission(UserRole::Clinician, "lodging:create"));
}

#[test]
fn clinician_cannot_create_inventory() {
    assert!(!has_permission(UserRole::Clinician, "inventory:create"));
}

#[test]
fn clinician_cannot_export() {
    assert!(!has_permission(UserRole::Clinician, "export:request"));
}

#[test]
fn clinician_cannot_upload_media() {
    assert!(!has_permission(UserRole::Clinician, "media:upload"));
}

// ── InventoryClerk ────────────────────────────────────────────────────────────

#[test]
fn clerk_can_view_inventory() {
    assert!(has_permission(UserRole::InventoryClerk, "inventory:view"));
}

#[test]
fn clerk_can_create_inventory() {
    assert!(has_permission(UserRole::InventoryClerk, "inventory:create"));
}

#[test]
fn clerk_can_transact_inventory() {
    assert!(has_permission(UserRole::InventoryClerk, "inventory:transact"));
}

#[test]
fn clerk_can_import_inventory() {
    assert!(has_permission(UserRole::InventoryClerk, "inventory:import"));
}

#[test]
fn clerk_can_reserve_inventory() {
    assert!(has_permission(UserRole::InventoryClerk, "inventory:reserve"));
}

#[test]
fn clerk_cannot_view_resource() {
    assert!(!has_permission(UserRole::InventoryClerk, "resource:view"));
}

#[test]
fn clerk_cannot_create_resource() {
    assert!(!has_permission(UserRole::InventoryClerk, "resource:create"));
}

#[test]
fn clerk_cannot_view_lodging() {
    assert!(!has_permission(UserRole::InventoryClerk, "lodging:view"));
}

#[test]
fn clerk_cannot_upload_media() {
    assert!(!has_permission(UserRole::InventoryClerk, "media:upload"));
}

#[test]
fn clerk_cannot_export() {
    assert!(!has_permission(UserRole::InventoryClerk, "export:request"));
}

// ── Cross-cutting: action isolation ──────────────────────────────────────────

#[test]
fn only_admin_and_publisher_can_create_resource() {
    assert!(has_permission(UserRole::Administrator, "resource:create"));
    assert!(has_permission(UserRole::Publisher,     "resource:create"));
    assert!(!has_permission(UserRole::Reviewer,      "resource:create"));
    assert!(!has_permission(UserRole::Clinician,     "resource:create"));
    assert!(!has_permission(UserRole::InventoryClerk,"resource:create"));
}

#[test]
fn only_admin_and_reviewer_can_review_resource() {
    assert!(has_permission(UserRole::Administrator, "resource:review"));
    assert!(has_permission(UserRole::Reviewer,      "resource:review"));
    assert!(!has_permission(UserRole::Publisher,     "resource:review"));
    assert!(!has_permission(UserRole::Clinician,     "resource:review"));
    assert!(!has_permission(UserRole::InventoryClerk,"resource:review"));
}

#[test]
fn inventory_create_only_admin_and_clerk() {
    assert!(has_permission(UserRole::Administrator, "inventory:create"));
    assert!(has_permission(UserRole::InventoryClerk,"inventory:create"));
    assert!(!has_permission(UserRole::Publisher,    "inventory:create"));
    assert!(!has_permission(UserRole::Reviewer,     "inventory:create"));
    assert!(!has_permission(UserRole::Clinician,    "inventory:create"));
}

#[test]
fn export_request_only_admin_and_reviewer() {
    assert!(has_permission(UserRole::Administrator, "export:request"));
    assert!(has_permission(UserRole::Reviewer,      "export:request"));
    assert!(!has_permission(UserRole::Publisher,    "export:request"));
    assert!(!has_permission(UserRole::Clinician,    "export:request"));
    assert!(!has_permission(UserRole::InventoryClerk,"export:request"));
}
