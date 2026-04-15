//! Tests for PII masking utilities.
//!
//! Calls `frontend_logic::mask::{mask_phone, mask_email}` directly — the same
//! functions that `frontend/src/services/mask.rs` delegates to.

use frontend_logic::mask::{mask_phone, mask_email};

// ── mask_phone ────────────────────────────────────────────────────────────────

#[test]
fn mask_phone_standard_10_digit() {
    assert_eq!(mask_phone("4155551234"), "(415) ***-1234");
}

#[test]
fn mask_phone_with_formatting() {
    assert_eq!(mask_phone("(415) 555-1234"), "(415) ***-1234");
}

#[test]
fn mask_phone_with_country_code() {
    // 11-digit: country code + 10 = "(141) ***-1234"
    assert_eq!(mask_phone("14155551234"), "(141) ***-1234");
}

#[test]
fn mask_phone_with_dashes() {
    assert_eq!(mask_phone("415-555-1234"), "(415) ***-1234");
}

#[test]
fn mask_phone_with_spaces() {
    assert_eq!(mask_phone("415 555 1234"), "(415) ***-1234");
}

#[test]
fn mask_phone_too_short_returns_placeholder() {
    assert_eq!(mask_phone("123"), "***-****");
}

#[test]
fn mask_phone_empty_returns_placeholder() {
    assert_eq!(mask_phone(""), "***-****");
}

#[test]
fn mask_phone_exactly_10_digits() {
    assert_eq!(mask_phone("0000000000"), "(000) ***-0000");
}

// ── mask_email ────────────────────────────────────────────────────────────────

#[test]
fn mask_email_standard_address() {
    assert_eq!(mask_email("john@example.com"), "j***n@example.com");
}

#[test]
fn mask_email_single_char_local() {
    assert_eq!(mask_email("a@example.com"), "a***@example.com");
}

#[test]
fn mask_email_two_char_local() {
    assert_eq!(mask_email("ab@example.com"), "a***@example.com");
}

#[test]
fn mask_email_long_local() {
    assert_eq!(mask_email("administrator@example.com"), "a***r@example.com");
}

#[test]
fn mask_email_no_at_sign() {
    assert_eq!(mask_email("notanemail"), "***");
}

#[test]
fn mask_email_preserves_domain() {
    let masked = mask_email("user@mycompany.org");
    assert!(masked.ends_with("@mycompany.org"));
}

#[test]
fn mask_email_subdomain_preserved() {
    let masked = mask_email("x@mail.example.co.uk");
    assert!(masked.ends_with("@mail.example.co.uk"));
}
