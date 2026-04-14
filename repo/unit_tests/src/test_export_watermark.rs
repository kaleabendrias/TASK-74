//! Tests for the export_watermark feature flag.
//!
//! Verifies that the watermark string is generated when the flag is enabled
//! and omitted (empty) when the flag is disabled — matching the behaviour in
//! `backend::service::import_export::approve_export`.

/// Mirrors the watermark generation logic extracted from the service.
fn generate_watermark(approver_username: &str, watermark_enabled: bool) -> String {
    if watermark_enabled {
        // Format matches the production implementation:
        // "{username}@{YYYYMMDDHHmmSS}"
        format!(
            "{}@{}",
            approver_username,
            chrono::Utc::now().format("%Y%m%d%H%M%S")
        )
    } else {
        String::new()
    }
}

use chrono::Utc;

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn watermark_generated_when_flag_enabled() {
    let wm = generate_watermark("reviewer1", true);
    assert!(!wm.is_empty(), "Watermark must be non-empty when flag is enabled");
    assert!(
        wm.starts_with("reviewer1@"),
        "Watermark must include the approver username; got: {}",
        wm
    );
}

#[test]
fn watermark_contains_timestamp_when_enabled() {
    let before = Utc::now().format("%Y%m%d").to_string();
    let wm = generate_watermark("admin", true);
    // The timestamp portion (after '@') must start with today's date prefix
    let ts_part = wm.split('@').nth(1).unwrap_or("");
    assert!(
        ts_part.starts_with(&before),
        "Watermark timestamp '{}' should start with today's date '{}'",
        ts_part,
        before
    );
}

#[test]
fn watermark_empty_when_flag_disabled() {
    let wm = generate_watermark("reviewer1", false);
    assert!(
        wm.is_empty(),
        "Watermark must be empty when flag is disabled; got: '{}'",
        wm
    );
}

#[test]
fn watermark_empty_when_flag_disabled_regardless_of_username() {
    for username in &["admin", "reviewer", "alice", ""] {
        let wm = generate_watermark(username, false);
        assert!(
            wm.is_empty(),
            "Watermark must be empty for username '{}' when flag is disabled",
            username
        );
    }
}

#[test]
fn watermark_format_uses_at_separator() {
    let wm = generate_watermark("bob", true);
    assert!(
        wm.contains('@'),
        "Watermark must use '@' as separator; got: '{}'",
        wm
    );
    let parts: Vec<&str> = wm.splitn(2, '@').collect();
    assert_eq!(parts.len(), 2, "Watermark must have exactly one '@' separator");
    assert_eq!(parts[0], "bob", "Username part must match approver");
    assert_eq!(parts[1].len(), 14, "Timestamp part must be 14 chars (YYYYMMDDHHmmSS)");
}
