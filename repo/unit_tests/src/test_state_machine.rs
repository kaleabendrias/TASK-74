//! Tests for resource state transition rules, calling the production function
//! directly via tourism_backend::service::resources::validate_state_transition.

use tourism_backend::model::UserRole;
use tourism_backend::service::resources::validate_state_transition;

// ── Legal transitions ──
#[test]
fn draft_to_in_review_by_publisher() {
    assert!(validate_state_transition("draft", "in_review", UserRole::Publisher).is_ok());
}

#[test]
fn in_review_to_published_by_reviewer() {
    assert!(validate_state_transition("in_review", "published", UserRole::Reviewer).is_ok());
}

#[test]
fn published_to_offline_by_publisher() {
    assert!(validate_state_transition("published", "offline", UserRole::Publisher).is_ok());
}

#[test]
fn published_to_offline_by_administrator() {
    assert!(validate_state_transition("published", "offline", UserRole::Administrator).is_ok());
}

#[test]
fn offline_to_draft_by_publisher() {
    assert!(validate_state_transition("offline", "draft", UserRole::Publisher).is_ok());
}

// ── Illegal transitions ──
#[test]
fn draft_to_in_review_by_reviewer_denied() {
    assert!(validate_state_transition("draft", "in_review", UserRole::Reviewer).is_err());
}

#[test]
fn draft_to_in_review_by_clinician_denied() {
    assert!(validate_state_transition("draft", "in_review", UserRole::Clinician).is_err());
}

#[test]
fn in_review_to_published_by_publisher_denied() {
    assert!(validate_state_transition("in_review", "published", UserRole::Publisher).is_err());
}

#[test]
fn published_to_offline_by_reviewer_denied() {
    assert!(validate_state_transition("published", "offline", UserRole::Reviewer).is_err());
}

#[test]
fn published_to_offline_by_clinician_denied() {
    assert!(validate_state_transition("published", "offline", UserRole::Clinician).is_err());
}

#[test]
fn draft_to_published_skip_denied() {
    assert!(validate_state_transition("draft", "published", UserRole::Administrator).is_err());
}

#[test]
fn draft_to_offline_denied() {
    assert!(validate_state_transition("draft", "offline", UserRole::Administrator).is_err());
}

#[test]
fn in_review_to_draft_denied() {
    assert!(validate_state_transition("in_review", "draft", UserRole::Reviewer).is_err());
}

#[test]
fn offline_to_published_skip_denied() {
    assert!(validate_state_transition("offline", "published", UserRole::Publisher).is_err());
}

#[test]
fn offline_to_draft_by_reviewer_denied() {
    assert!(validate_state_transition("offline", "draft", UserRole::Reviewer).is_err());
}

#[test]
fn same_state_transition_denied() {
    assert!(validate_state_transition("draft", "draft", UserRole::Publisher).is_err());
    assert!(validate_state_transition("published", "published", UserRole::Administrator).is_err());
}

#[test]
fn inventory_clerk_denied_all() {
    assert!(validate_state_transition("draft", "in_review", UserRole::InventoryClerk).is_err());
    assert!(validate_state_transition("in_review", "published", UserRole::InventoryClerk).is_err());
    assert!(validate_state_transition("published", "offline", UserRole::InventoryClerk).is_err());
    assert!(validate_state_transition("offline", "draft", UserRole::InventoryClerk).is_err());
}

// ── Full lifecycle ──
#[test]
fn full_lifecycle() {
    assert!(validate_state_transition("draft", "in_review", UserRole::Publisher).is_ok());
    assert!(validate_state_transition("in_review", "published", UserRole::Reviewer).is_ok());
    assert!(validate_state_transition("published", "offline", UserRole::Administrator).is_ok());
    assert!(validate_state_transition("offline", "draft", UserRole::Publisher).is_ok());
}
