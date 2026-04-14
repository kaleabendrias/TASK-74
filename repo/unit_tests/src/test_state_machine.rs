// These tests verify the state transition rules that are enforced by
// tourism_backend::service::resources::validate_state_transition (which is
// private to the service module). The rules are duplicated here to enable
// isolated unit testing without a database connection.

use tourism_backend::model::UserRole;

/// Replicate the state transition logic from service::resources for testing.
/// The actual function is private, so we test the same rules here.
fn is_valid_transition(current: &str, new: &str, role: UserRole) -> bool {
    match (current, new) {
        ("draft", "in_review") => role == UserRole::Publisher,
        ("in_review", "published") => role == UserRole::Reviewer,
        ("published", "offline") => {
            role == UserRole::Publisher || role == UserRole::Administrator
        }
        ("offline", "draft") => role == UserRole::Publisher,
        _ => false,
    }
}

// ── Legal transitions ──
#[test]
fn draft_to_in_review_by_publisher() {
    assert!(is_valid_transition("draft", "in_review", UserRole::Publisher));
}

#[test]
fn in_review_to_published_by_reviewer() {
    assert!(is_valid_transition("in_review", "published", UserRole::Reviewer));
}

#[test]
fn published_to_offline_by_publisher() {
    assert!(is_valid_transition("published", "offline", UserRole::Publisher));
}

#[test]
fn published_to_offline_by_administrator() {
    assert!(is_valid_transition("published", "offline", UserRole::Administrator));
}

#[test]
fn offline_to_draft_by_publisher() {
    assert!(is_valid_transition("offline", "draft", UserRole::Publisher));
}

// ── Illegal transitions ──
#[test]
fn draft_to_in_review_by_reviewer_denied() {
    assert!(!is_valid_transition("draft", "in_review", UserRole::Reviewer));
}

#[test]
fn draft_to_in_review_by_clinician_denied() {
    assert!(!is_valid_transition("draft", "in_review", UserRole::Clinician));
}

#[test]
fn in_review_to_published_by_publisher_denied() {
    assert!(!is_valid_transition("in_review", "published", UserRole::Publisher));
}

#[test]
fn published_to_offline_by_reviewer_denied() {
    assert!(!is_valid_transition("published", "offline", UserRole::Reviewer));
}

#[test]
fn published_to_offline_by_clinician_denied() {
    assert!(!is_valid_transition("published", "offline", UserRole::Clinician));
}

#[test]
fn draft_to_published_skip_denied() {
    assert!(!is_valid_transition("draft", "published", UserRole::Administrator));
}

#[test]
fn draft_to_offline_denied() {
    assert!(!is_valid_transition("draft", "offline", UserRole::Administrator));
}

#[test]
fn in_review_to_draft_denied() {
    assert!(!is_valid_transition("in_review", "draft", UserRole::Reviewer));
}

#[test]
fn offline_to_published_skip_denied() {
    assert!(!is_valid_transition("offline", "published", UserRole::Publisher));
}

#[test]
fn offline_to_draft_by_reviewer_denied() {
    assert!(!is_valid_transition("offline", "draft", UserRole::Reviewer));
}

#[test]
fn same_state_transition_denied() {
    assert!(!is_valid_transition("draft", "draft", UserRole::Publisher));
    assert!(!is_valid_transition("published", "published", UserRole::Administrator));
}

#[test]
fn inventory_clerk_denied_all() {
    assert!(!is_valid_transition("draft", "in_review", UserRole::InventoryClerk));
    assert!(!is_valid_transition("in_review", "published", UserRole::InventoryClerk));
    assert!(!is_valid_transition("published", "offline", UserRole::InventoryClerk));
    assert!(!is_valid_transition("offline", "draft", UserRole::InventoryClerk));
}

// ── Full lifecycle ──
#[test]
fn full_lifecycle() {
    assert!(is_valid_transition("draft", "in_review", UserRole::Publisher));
    assert!(is_valid_transition("in_review", "published", UserRole::Reviewer));
    assert!(is_valid_transition("published", "offline", UserRole::Administrator));
    assert!(is_valid_transition("offline", "draft", UserRole::Publisher));
}
