//! Tests for inventory input validation, calling production functions from
//! tourism_backend::service::inventory.

use tourism_backend::service::inventory::{validate_reserve_input, validate_transaction_input};

// ── Reserve validation ──

#[test]
fn reserve_positive_quantity_ok() {
    assert!(validate_reserve_input(1).is_ok());
    assert!(validate_reserve_input(100).is_ok());
}

#[test]
fn reserve_zero_quantity_fails() {
    let err = validate_reserve_input(0).unwrap_err();
    assert_eq!(err.body.code, "VALIDATION_ERROR");
}

#[test]
fn reserve_negative_quantity_fails() {
    assert!(validate_reserve_input(-1).is_err());
}

// ── Transaction validation ──

#[test]
fn transaction_inbound_ok() {
    assert!(validate_transaction_input("inbound", 10).is_ok());
}

#[test]
fn transaction_outbound_ok() {
    assert!(validate_transaction_input("outbound", 5).is_ok());
}

#[test]
fn transaction_invalid_direction_fails() {
    let err = validate_transaction_input("sideways", 10).unwrap_err();
    assert_eq!(err.body.code, "VALIDATION_ERROR");
    assert!(err.body.message.contains("Direction"));
}

#[test]
fn transaction_zero_quantity_fails() {
    assert!(validate_transaction_input("inbound", 0).is_err());
}

#[test]
fn transaction_negative_quantity_fails() {
    assert!(validate_transaction_input("outbound", -1).is_err());
}
