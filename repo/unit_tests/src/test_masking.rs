//! Tests for PII masking functions, calling the production implementations
//! in tourism_backend::service::masking.

use tourism_backend::service::masking::{mask_phone, mask_email};

#[test]
fn phone_standard_10_digit() {
    assert_eq!(mask_phone("4155551234"), "(415) ***-1234");
}

#[test]
fn phone_with_formatting() {
    assert_eq!(mask_phone("(415) 555-1234"), "(415) ***-1234");
}

#[test]
fn phone_with_country_code() {
    assert_eq!(mask_phone("+1-415-555-1234"), "(141) ***-1234");
}

#[test]
fn phone_short_number() {
    assert_eq!(mask_phone("12345"), "***-****");
}

#[test]
fn phone_empty() {
    assert_eq!(mask_phone(""), "***-****");
}

#[test]
fn email_normal() {
    assert_eq!(mask_email("john@example.com"), "j***n@example.com");
}

#[test]
fn email_short_local() {
    assert_eq!(mask_email("ab@example.com"), "a***@example.com");
}

#[test]
fn email_single_char() {
    assert_eq!(mask_email("a@example.com"), "a***@example.com");
}

#[test]
fn email_no_at() {
    assert_eq!(mask_email("noemail"), "***");
}

#[test]
fn email_long_local() {
    assert_eq!(mask_email("longusername@example.com"), "l***e@example.com");
}

#[test]
fn email_with_dots() {
    assert_eq!(mask_email("first.last@example.com"), "f***t@example.com");
}
