//! Tests for client-side validation logic.
//!
//! Calls `frontend_logic::validation` functions directly — the same functions
//! that the Yew page components (login.rs, lodgings.rs, resources.rs,
//! inventory.rs) use before submitting API requests.

use frontend_logic::validation::{
    validate_login, validate_deposit_cap, validate_period_nights,
    validate_lot_quantity, validate_rent_change, validate_resource_title,
};

// ── Login validation ──────────────────────────────────────────────────────────

#[test]
fn login_valid_credentials_produce_no_errors() {
    assert!(validate_login("admin", "pass1234").is_empty());
}

#[test]
fn login_empty_username_produces_error() {
    let e = validate_login("", "password");
    assert!(e.iter().any(|(f, _)| *f == "username"));
}

#[test]
fn login_empty_password_produces_error() {
    let e = validate_login("admin", "");
    assert!(e.iter().any(|(f, _)| *f == "password"));
}

#[test]
fn login_short_password_produces_error() {
    let e = validate_login("admin", "abc");
    assert!(e.iter().any(|(_, m)| m.contains("4 characters")));
}

#[test]
fn login_exactly_four_char_password_is_valid() {
    assert!(validate_login("admin", "pass").is_empty());
}

#[test]
fn login_both_empty_produces_two_errors() {
    assert_eq!(validate_login("", "").len(), 2);
}

// ── Deposit cap ───────────────────────────────────────────────────────────────

#[test]
fn deposit_exactly_1_5x_rent_is_allowed() {
    assert!(validate_deposit_cap(1500.0, 1000.0));
}

#[test]
fn deposit_below_cap_is_allowed() {
    assert!(validate_deposit_cap(1000.0, 1000.0));
    assert!(validate_deposit_cap(500.0,  1000.0));
}

#[test]
fn deposit_above_1_5x_rent_is_rejected() {
    assert!(!validate_deposit_cap(1501.0, 1000.0));
    assert!(!validate_deposit_cap(3000.0, 1000.0));
}

#[test]
fn deposit_cap_zero_rent_edge_case() {
    assert!(validate_deposit_cap(0.0, 0.0));
}

// ── Period nights ─────────────────────────────────────────────────────────────

#[test]
fn valid_period_7_to_30_produces_no_errors() {
    assert!(validate_period_nights(7, 30).is_empty());
}

#[test]
fn min_below_7_is_invalid() {
    let e = validate_period_nights(6, 30);
    assert!(e.iter().any(|m| m.contains("min_nights must be at least 7")));
}

#[test]
fn max_above_365_is_invalid() {
    let e = validate_period_nights(7, 366);
    assert!(e.iter().any(|m| m.contains("max_nights must not exceed 365")));
}

#[test]
fn min_greater_than_max_is_invalid() {
    let e = validate_period_nights(30, 7);
    assert!(e.iter().any(|m| m.contains("min_nights must not exceed max_nights")));
}

#[test]
fn period_exactly_365_max_is_valid() {
    assert!(validate_period_nights(7, 365).is_empty());
}

#[test]
fn period_min_equals_max_is_valid() {
    assert!(validate_period_nights(14, 14).is_empty());
}

// ── Lot quantity ──────────────────────────────────────────────────────────────

#[test]
fn positive_lot_quantity_is_valid() {
    assert!(validate_lot_quantity(1));
    assert!(validate_lot_quantity(1000));
}

#[test]
fn zero_lot_quantity_is_invalid() {
    assert!(!validate_lot_quantity(0));
}

#[test]
fn negative_lot_quantity_is_invalid() {
    assert!(!validate_lot_quantity(-1));
}

// ── Rent change values ────────────────────────────────────────────────────────

#[test]
fn positive_rent_and_deposit_are_valid() {
    assert!(validate_rent_change(1000.0, 1000.0));
}

#[test]
fn zero_rent_is_invalid() {
    assert!(!validate_rent_change(0.0, 1000.0));
}

#[test]
fn zero_deposit_is_invalid() {
    assert!(!validate_rent_change(1000.0, 0.0));
}

// ── Resource title ────────────────────────────────────────────────────────────

#[test]
fn non_empty_title_under_200_chars_is_valid() {
    assert!(validate_resource_title("Park Trail Guide"));
}

#[test]
fn empty_title_is_invalid() {
    assert!(!validate_resource_title(""));
}

#[test]
fn title_of_exactly_200_chars_is_valid() {
    assert!(validate_resource_title(&"A".repeat(200)));
}

#[test]
fn title_of_201_chars_is_invalid() {
    assert!(!validate_resource_title(&"A".repeat(201)));
}
