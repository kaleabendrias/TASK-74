use tourism_backend::service::validation;

// ── Title ──
#[test]
fn title_valid() {
    assert!(validation::validate_title("My Resource").is_ok());
}

#[test]
fn title_empty_fails() {
    let err = validation::validate_title("").unwrap_err();
    assert_eq!(err.field, "title");
    assert!(err.message.contains("required"));
}

#[test]
fn title_exactly_200_chars_ok() {
    let title = "a".repeat(200);
    assert!(validation::validate_title(&title).is_ok());
}

#[test]
fn title_201_chars_fails() {
    let title = "a".repeat(201);
    let err = validation::validate_title(&title).unwrap_err();
    assert_eq!(err.field, "title");
    assert!(err.message.contains("200"));
}

// ── Tags ──
#[test]
fn tags_valid_20() {
    let tags: Vec<String> = (0..20).map(|i| format!("tag{}", i)).collect();
    assert!(validation::validate_tags(&tags).is_ok());
}

#[test]
fn tags_21_fails() {
    let tags: Vec<String> = (0..21).map(|i| format!("tag{}", i)).collect();
    let err = validation::validate_tags(&tags).unwrap_err();
    assert_eq!(err.field, "tags");
}

#[test]
fn tags_empty_ok() {
    assert!(validation::validate_tags(&[]).is_ok());
}

// ── Pricing ──
#[test]
fn pricing_valid() {
    let p = serde_json::json!({"adult": 10.0, "child": 5.0});
    assert!(validation::validate_pricing(&p).is_ok());
}

#[test]
fn pricing_zero_ok() {
    let p = serde_json::json!({"free": 0.0});
    assert!(validation::validate_pricing(&p).is_ok());
}

#[test]
fn pricing_negative_fails() {
    let p = serde_json::json!({"adult": -1.0});
    let err = validation::validate_pricing(&p).unwrap_err();
    assert!(err.field.contains("pricing"));
    assert!(err.message.contains("non-negative"));
}

#[test]
fn pricing_null_ok() {
    assert!(validation::validate_pricing(&serde_json::Value::Null).is_ok());
}

// ── Lat/Lng ──
#[test]
fn lat_lng_valid() {
    assert!(validation::validate_lat_lng(Some(45.0), Some(-122.0)).is_ok());
}

#[test]
fn lat_lng_boundaries() {
    assert!(validation::validate_lat_lng(Some(90.0), Some(180.0)).is_ok());
    assert!(validation::validate_lat_lng(Some(-90.0), Some(-180.0)).is_ok());
}

#[test]
fn lat_out_of_range() {
    let errs = validation::validate_lat_lng(Some(91.0), None).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].field, "latitude");
}

#[test]
fn lng_out_of_range() {
    let errs = validation::validate_lat_lng(None, Some(-181.0)).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].field, "longitude");
}

#[test]
fn both_out_of_range() {
    let errs = validation::validate_lat_lng(Some(100.0), Some(200.0)).unwrap_err();
    assert_eq!(errs.len(), 2);
}

#[test]
fn lat_lng_none_ok() {
    assert!(validation::validate_lat_lng(None, None).is_ok());
}

// ── Hours ──
#[test]
fn hours_object_ok() {
    let h = serde_json::json!({"monday": {"open": "09:00", "close": "17:00"}});
    assert!(validation::validate_hours(&h).is_ok());
}

#[test]
fn hours_null_ok() {
    assert!(validation::validate_hours(&serde_json::Value::Null).is_ok());
}

#[test]
fn hours_array_fails() {
    let h = serde_json::json!([1, 2, 3]);
    let err = validation::validate_hours(&h).unwrap_err();
    assert_eq!(err.field, "hours");
}

// ── Amenities ──
#[test]
fn amenities_valid() {
    let a = vec!["wifi".to_string(), "pool".to_string()];
    assert!(validation::validate_amenities(&a).is_ok());
}

#[test]
fn amenities_unknown_fails() {
    let a = vec!["wifi".to_string(), "hot_tub".to_string()];
    let errs = validation::validate_amenities(&a).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("hot_tub"));
}

#[test]
fn amenities_all_allowed() {
    let a: Vec<String> = validation::ALLOWED_AMENITIES.iter().map(|s| s.to_string()).collect();
    assert!(validation::validate_amenities(&a).is_ok());
}

#[test]
fn amenities_empty_ok() {
    assert!(validation::validate_amenities(&[]).is_ok());
}
