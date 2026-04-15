//! Tests for scheduled_publish_at timezone conversion.
//!
//! Exercises `tourism_backend::service::resources::parse_scheduled_publish` directly.
//!
//! The function converts a naive local datetime plus a `tz_offset_minutes` value into
//! UTC. The offset follows the ISO / chrono sign convention: positive = east of UTC
//! (e.g. UTC+5 → +300), negative = west (e.g. UTC-8 → -480).

use chrono::{TimeZone, Utc};
use tourism_backend::service::resources::parse_scheduled_publish;

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn no_input_returns_none() {
    let result = parse_scheduled_publish(&None, None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn empty_string_returns_none() {
    let result = parse_scheduled_publish(&Some(String::new()), None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn invalid_datetime_string_returns_error() {
    let result = parse_scheduled_publish(&Some("not-a-date".to_string()), None);
    assert!(result.is_err(), "Expected error for invalid datetime");
}

/// When no offset is provided the naive datetime is treated as UTC.
#[test]
fn no_offset_treated_as_utc() {
    let input = Some("2025-06-15T14:00:00".to_string());
    let result = parse_scheduled_publish(&input, None).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 15, 14, 0, 0).unwrap();
    assert_eq!(result, expected);
}

/// UTC+0 offset leaves the datetime unchanged.
#[test]
fn utc_plus_zero_no_change() {
    let input = Some("2025-06-15T14:00:00".to_string());
    let result = parse_scheduled_publish(&input, Some(0)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 15, 14, 0, 0).unwrap();
    assert_eq!(result, expected);
}

/// UTC+5 (offset = +300 minutes): 14:00 local → 09:00 UTC.
#[test]
fn utc_plus_5_subtracts_hours() {
    let input = Some("2025-06-15T14:00:00".to_string());
    let result = parse_scheduled_publish(&input, Some(300)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 15, 9, 0, 0).unwrap();
    assert_eq!(result, expected);
}

/// UTC+5:30 (India Standard Time, +330 minutes): 14:00 local → 08:30 UTC.
#[test]
fn utc_plus_5_30_ist() {
    let input = Some("2025-06-15T14:00:00".to_string());
    let result = parse_scheduled_publish(&input, Some(330)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 15, 8, 30, 0).unwrap();
    assert_eq!(result, expected);
}

/// UTC-8 (Pacific Standard Time, -480 minutes): 14:00 local → 22:00 UTC same day.
#[test]
fn utc_minus_8_pst() {
    let input = Some("2025-06-15T14:00:00".to_string());
    let result = parse_scheduled_publish(&input, Some(-480)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 15, 22, 0, 0).unwrap();
    assert_eq!(result, expected);
}

/// Midnight crossing: 23:00 local UTC+2 → 21:00 UTC same day.
#[test]
fn midnight_boundary_utc_plus_2() {
    let input = Some("2025-06-15T23:00:00".to_string());
    let result = parse_scheduled_publish(&input, Some(120)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 15, 21, 0, 0).unwrap();
    assert_eq!(result, expected);
}

/// Midnight crossing forward: 23:00 local UTC-3 → 02:00 UTC next day.
#[test]
fn midnight_crossing_next_day_utc_minus_3() {
    let input = Some("2025-06-15T23:00:00".to_string());
    let result = parse_scheduled_publish(&input, Some(-180)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 16, 2, 0, 0).unwrap();
    assert_eq!(result, expected);
}

/// Large positive offset (UTC+14, e.g. Line Islands): 01:00 local → 11:00 UTC previous day.
#[test]
fn utc_plus_14_line_islands() {
    let input = Some("2025-06-16T01:00:00".to_string());
    let result = parse_scheduled_publish(&input, Some(840)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 6, 15, 11, 0, 0).unwrap();
    assert_eq!(result, expected);
}

/// Without-seconds format "YYYY-MM-DDTHH:MM" is also accepted.
#[test]
fn datetime_without_seconds_parsed() {
    let input = Some("2025-12-31T23:59".to_string());
    let result = parse_scheduled_publish(&input, Some(0)).unwrap().unwrap();
    let expected = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 0).unwrap();
    assert_eq!(result, expected);
}
