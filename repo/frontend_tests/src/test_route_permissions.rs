//! Tests for the RouteGuard permission matrix.
//!
//! Calls `frontend_logic::routing::can_access` directly — the same function
//! that mirrors the `allowed_roles` configuration in the frontend's `app.rs`.

use frontend_logic::routing::{Route, can_access};
use frontend_logic::models::UserRole;

#[test]
fn admin_can_access_all_routes() {
    for route in [
        Route::Dashboard, Route::ResourceList, Route::ResourceNew,
        Route::LodgingList, Route::LodgingNew, Route::Inventory,
        Route::InventoryTransactions, Route::ImportExport,
        Route::Configuration, Route::SecuritySettings,
    ] {
        assert!(can_access(&UserRole::Administrator, &route),
            "Admin must access {:?}", route);
    }
}

#[test]
fn publisher_cannot_access_inventory_or_config() {
    assert!(!can_access(&UserRole::Publisher, &Route::Inventory));
    assert!(!can_access(&UserRole::Publisher, &Route::InventoryTransactions));
    assert!(!can_access(&UserRole::Publisher, &Route::ImportExport));
    assert!(!can_access(&UserRole::Publisher, &Route::Configuration));
}

#[test]
fn publisher_can_access_content_routes() {
    assert!(can_access(&UserRole::Publisher, &Route::ResourceList));
    assert!(can_access(&UserRole::Publisher, &Route::ResourceNew));
    assert!(can_access(&UserRole::Publisher, &Route::LodgingList));
    assert!(can_access(&UserRole::Publisher, &Route::LodgingNew));
    assert!(can_access(&UserRole::Publisher, &Route::Dashboard));
}

#[test]
fn reviewer_cannot_create_resources_or_access_config() {
    assert!(!can_access(&UserRole::Reviewer, &Route::ResourceNew));
    assert!(!can_access(&UserRole::Reviewer, &Route::LodgingNew));
    assert!(!can_access(&UserRole::Reviewer, &Route::Configuration));
    assert!(!can_access(&UserRole::Reviewer, &Route::Inventory));
}

#[test]
fn reviewer_can_access_content_and_data() {
    assert!(can_access(&UserRole::Reviewer, &Route::ResourceList));
    assert!(can_access(&UserRole::Reviewer, &Route::LodgingList));
    assert!(can_access(&UserRole::Reviewer, &Route::ImportExport));
    assert!(can_access(&UserRole::Reviewer, &Route::Dashboard));
}

#[test]
fn clinician_cannot_write_resources_or_access_data_system() {
    assert!(!can_access(&UserRole::Clinician, &Route::ResourceNew));
    assert!(!can_access(&UserRole::Clinician, &Route::LodgingNew));
    assert!(!can_access(&UserRole::Clinician, &Route::ImportExport));
    assert!(!can_access(&UserRole::Clinician, &Route::Configuration));
}

#[test]
fn clinician_can_view_content_and_inventory() {
    assert!(can_access(&UserRole::Clinician, &Route::ResourceList));
    assert!(can_access(&UserRole::Clinician, &Route::LodgingList));
    assert!(can_access(&UserRole::Clinician, &Route::Inventory));
    assert!(can_access(&UserRole::Clinician, &Route::Dashboard));
}

#[test]
fn inventory_clerk_cannot_access_content_write_or_config() {
    assert!(!can_access(&UserRole::InventoryClerk, &Route::ResourceNew));
    assert!(!can_access(&UserRole::InventoryClerk, &Route::LodgingNew));
    assert!(!can_access(&UserRole::InventoryClerk, &Route::Configuration));
}

#[test]
fn inventory_clerk_can_access_inventory_and_data() {
    assert!(can_access(&UserRole::InventoryClerk, &Route::Inventory));
    assert!(can_access(&UserRole::InventoryClerk, &Route::InventoryTransactions));
    assert!(can_access(&UserRole::InventoryClerk, &Route::ImportExport));
    assert!(can_access(&UserRole::InventoryClerk, &Route::Dashboard));
}

#[test]
fn configuration_exclusive_to_administrator() {
    assert!(can_access(&UserRole::Administrator, &Route::Configuration));
    for role in [
        UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ] {
        assert!(!can_access(&role, &Route::Configuration),
            "{:?} must not access Configuration", role);
    }
}

#[test]
fn security_settings_accessible_to_all_roles() {
    for role in [
        UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ] {
        assert!(can_access(&role, &Route::SecuritySettings),
            "{:?} must access SecuritySettings", role);
    }
}

#[test]
fn dashboard_accessible_to_all_roles() {
    for role in [
        UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ] {
        assert!(can_access(&role, &Route::Dashboard),
            "{:?} must access Dashboard", role);
    }
}
