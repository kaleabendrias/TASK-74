//! Tests for the sidebar role-visibility matrix.
//!
//! Calls `frontend_logic::sidebar::visible_sections` directly — the same
//! function that `frontend/src/components/sidebar.rs` uses to decide which
//! nav sections to render.  Any change to the production logic immediately
//! breaks the matching tests here.

use frontend_logic::sidebar::visible_sections;
use frontend_logic::models::UserRole;

// ── Per-role visibility ───────────────────────────────────────────────────────

#[test]
fn administrator_sees_all_sections() {
    let s = visible_sections(&UserRole::Administrator);
    assert!(s.contains(&"Content"),   "Admin must see Content");
    assert!(s.contains(&"Inventory"), "Admin must see Inventory");
    assert!(s.contains(&"Data"),      "Admin must see Data");
    assert!(s.contains(&"System"),    "Admin must see System");
    assert!(s.contains(&"Account"),   "Admin must see Account");
}

#[test]
fn publisher_sees_content_and_account_only() {
    let s = visible_sections(&UserRole::Publisher);
    assert!(s.contains(&"Content"),     "Publisher must see Content");
    assert!(!s.contains(&"Inventory"),  "Publisher must NOT see Inventory");
    assert!(!s.contains(&"Data"),       "Publisher must NOT see Data");
    assert!(!s.contains(&"System"),     "Publisher must NOT see System");
    assert!(s.contains(&"Account"),     "Publisher must see Account");
}

#[test]
fn reviewer_sees_content_data_account() {
    let s = visible_sections(&UserRole::Reviewer);
    assert!(s.contains(&"Content"),     "Reviewer must see Content");
    assert!(!s.contains(&"Inventory"),  "Reviewer must NOT see Inventory");
    assert!(s.contains(&"Data"),        "Reviewer must see Data");
    assert!(!s.contains(&"System"),     "Reviewer must NOT see System");
    assert!(s.contains(&"Account"),     "Reviewer must see Account");
}

#[test]
fn clinician_sees_content_inventory_account() {
    let s = visible_sections(&UserRole::Clinician);
    assert!(s.contains(&"Content"),    "Clinician must see Content");
    assert!(s.contains(&"Inventory"),  "Clinician must see Inventory");
    assert!(!s.contains(&"Data"),      "Clinician must NOT see Data");
    assert!(!s.contains(&"System"),    "Clinician must NOT see System");
    assert!(s.contains(&"Account"),    "Clinician must see Account");
}

#[test]
fn inventory_clerk_sees_inventory_data_account() {
    let s = visible_sections(&UserRole::InventoryClerk);
    assert!(!s.contains(&"Content"),   "InventoryClerk must NOT see Content");
    assert!(s.contains(&"Inventory"),  "InventoryClerk must see Inventory");
    assert!(s.contains(&"Data"),       "InventoryClerk must see Data");
    assert!(!s.contains(&"System"),    "InventoryClerk must NOT see System");
    assert!(s.contains(&"Account"),    "InventoryClerk must see Account");
}

// ── Cross-role assertions ────────────────────────────────────────────────────

#[test]
fn account_section_visible_for_every_role() {
    for role in [
        UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ] {
        assert!(visible_sections(&role).contains(&"Account"),
            "{:?} must always see Account", role);
    }
}

#[test]
fn system_section_exclusive_to_administrator() {
    assert!(visible_sections(&UserRole::Administrator).contains(&"System"));
    for role in [
        UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ] {
        assert!(!visible_sections(&role).contains(&"System"),
            "{:?} must NOT see System", role);
    }
}

#[test]
fn inventory_not_visible_to_publisher_or_reviewer() {
    assert!(!visible_sections(&UserRole::Publisher).contains(&"Inventory"));
    assert!(!visible_sections(&UserRole::Reviewer).contains(&"Inventory"));
}

#[test]
fn data_not_visible_to_publisher_or_clinician() {
    assert!(!visible_sections(&UserRole::Publisher).contains(&"Data"));
    assert!(!visible_sections(&UserRole::Clinician).contains(&"Data"));
}

#[test]
fn content_not_visible_to_inventory_clerk() {
    assert!(!visible_sections(&UserRole::InventoryClerk).contains(&"Content"));
}

#[test]
fn inventory_section_visible_to_correct_three_roles() {
    let count = [
        UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ]
    .iter()
    .filter(|r| visible_sections(r).contains(&"Inventory"))
    .count();
    assert_eq!(count, 3, "Inventory visible to exactly 3 roles");
}

#[test]
fn data_section_visible_to_correct_three_roles() {
    let count = [
        UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ]
    .iter()
    .filter(|r| visible_sections(r).contains(&"Data"))
    .count();
    assert_eq!(count, 3, "Data visible to exactly 3 roles");
}

#[test]
fn content_section_visible_to_four_roles() {
    let count = [
        UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer,
        UserRole::Clinician, UserRole::InventoryClerk,
    ]
    .iter()
    .filter(|r| visible_sections(r).contains(&"Content"))
    .count();
    assert_eq!(count, 4, "Content visible to exactly 4 roles");
}
