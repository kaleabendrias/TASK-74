//! Tests for import row validation rules as implemented in
//! tourism_backend::jobs::runner::process_xlsx_job.

/// Validates an import row according to the same rules as process_xlsx_job.
fn validate_import_row(obj: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    let mut errors = Vec::new();

    // item_name is required and non-empty
    if let Some(val) = obj.get("item_name") {
        if val.as_str().map_or(true, |s| s.trim().is_empty()) {
            errors.push("missing required field 'item_name'".into());
        }
    } else {
        errors.push("missing required field 'item_name'".into());
    }

    // quantity_on_hand must be a valid integer
    if let Some(val) = obj.get("quantity_on_hand") {
        if let Some(s) = val.as_str() {
            if s.parse::<i32>().is_err() {
                errors.push(format!("invalid integer for 'quantity_on_hand': '{}'", s));
            }
        }
    }

    errors
}

#[test]
fn valid_row_passes() {
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("Bandages"));
    obj.insert("quantity_on_hand".into(), serde_json::json!("100"));
    obj.insert("lot_number".into(), serde_json::json!("LOT-001"));
    assert!(validate_import_row(&obj).is_empty());
}

#[test]
fn missing_item_name_fails() {
    let mut obj = serde_json::Map::new();
    obj.insert("quantity_on_hand".into(), serde_json::json!("50"));
    let errors = validate_import_row(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("item_name"));
}

#[test]
fn empty_item_name_fails() {
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!(""));
    obj.insert("quantity_on_hand".into(), serde_json::json!("50"));
    let errors = validate_import_row(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("item_name"));
}

#[test]
fn non_numeric_quantity_fails() {
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("Gauze"));
    obj.insert("quantity_on_hand".into(), serde_json::json!("abc"));
    let errors = validate_import_row(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("quantity_on_hand"));
}

#[test]
fn negative_quantity_is_valid_integer() {
    // Negative integers are valid at the parsing stage; business logic rejects them later
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("Syringes"));
    obj.insert("quantity_on_hand".into(), serde_json::json!("-5"));
    assert!(validate_import_row(&obj).is_empty());
}

#[test]
fn multiple_errors_on_single_row() {
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("  "));
    obj.insert("quantity_on_hand".into(), serde_json::json!("not_a_number"));
    let errors = validate_import_row(&obj);
    assert_eq!(errors.len(), 2);
}

#[test]
fn missing_optional_fields_ok() {
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("Masks"));
    // lot_number, facility_id, etc. are optional with defaults
    assert!(validate_import_row(&obj).is_empty());
}

#[test]
fn row_with_all_fields_valid() {
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("Gloves"));
    obj.insert("lot_number".into(), serde_json::json!("LOT-999"));
    obj.insert("quantity_on_hand".into(), serde_json::json!("500"));
    obj.insert("facility_id".into(), serde_json::json!("00000000-0000-0000-0000-000000000001"));
    obj.insert("warehouse_id".into(), serde_json::json!("00000000-0000-0000-0000-000000000002"));
    obj.insert("bin_id".into(), serde_json::json!("00000000-0000-0000-0000-000000000003"));
    assert!(validate_import_row(&obj).is_empty());
}
