//! Tests for import row validation rules.
//!
//! Exercises `tourism_backend::jobs::runner::validate_import_row_fields` directly,
//! verifying that the production function enforces all required field rules.

use tourism_backend::jobs::runner::validate_import_row_fields;

fn valid_obj() -> serde_json::Map<String, serde_json::Value> {
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("Bandages"));
    obj.insert("quantity_on_hand".into(), serde_json::json!("100"));
    obj.insert("lot_number".into(), serde_json::json!("LOT-001"));
    obj.insert("facility_id".into(), serde_json::json!("00000000-0000-0000-0000-000000000001"));
    obj.insert("warehouse_id".into(), serde_json::json!("00000000-0000-0000-0000-000000000002"));
    obj.insert("bin_id".into(), serde_json::json!("00000000-0000-0000-0000-000000000003"));
    obj
}

#[test]
fn valid_row_passes() {
    assert!(validate_import_row_fields(&valid_obj()).is_empty());
}

#[test]
fn missing_item_name_fails() {
    let mut obj = valid_obj();
    obj.remove("item_name");
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("item_name"));
}

#[test]
fn empty_item_name_fails() {
    let mut obj = valid_obj();
    obj.insert("item_name".into(), serde_json::json!(""));
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("item_name"));
}

#[test]
fn non_numeric_quantity_fails() {
    let mut obj = valid_obj();
    obj.insert("quantity_on_hand".into(), serde_json::json!("abc"));
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("quantity_on_hand"));
}

#[test]
fn negative_quantity_is_valid_integer() {
    // Negative integers are valid at the parsing stage; business logic rejects them later
    let mut obj = valid_obj();
    obj.insert("quantity_on_hand".into(), serde_json::json!("-5"));
    assert!(validate_import_row_fields(&obj).is_empty());
}

#[test]
fn multiple_errors_on_single_row() {
    // Production code pre-trims values before calling this function, so
    // we pass already-trimmed empty string (not whitespace) for item_name.
    let mut obj = valid_obj();
    obj.insert("item_name".into(), serde_json::json!(""));
    obj.insert("quantity_on_hand".into(), serde_json::json!("not_a_number"));
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 2, "expected 2 errors (empty item_name + bad qty), got {:?}", errors);
}

#[test]
fn missing_facility_id_fails() {
    let mut obj = valid_obj();
    obj.remove("facility_id");
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("facility_id"));
}

#[test]
fn missing_warehouse_id_fails() {
    let mut obj = valid_obj();
    obj.remove("warehouse_id");
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("warehouse_id"));
}

#[test]
fn missing_bin_id_fails() {
    let mut obj = valid_obj();
    obj.remove("bin_id");
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("bin_id"));
}

#[test]
fn invalid_uuid_for_facility_id_fails() {
    let mut obj = valid_obj();
    obj.insert("facility_id".into(), serde_json::json!("not-a-uuid"));
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("facility_id"));
}

#[test]
fn invalid_uuid_for_warehouse_id_fails() {
    let mut obj = valid_obj();
    obj.insert("warehouse_id".into(), serde_json::json!("bad-uuid"));
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("warehouse_id"));
}

#[test]
fn empty_facility_id_fails() {
    let mut obj = valid_obj();
    obj.insert("facility_id".into(), serde_json::json!(""));
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("facility_id"));
}

#[test]
fn all_location_fields_missing_produces_three_errors() {
    let mut obj = valid_obj();
    obj.remove("facility_id");
    obj.remove("warehouse_id");
    obj.remove("bin_id");
    let errors = validate_import_row_fields(&obj);
    assert_eq!(errors.len(), 3, "Expected 3 errors for all missing location fields, got {:?}", errors);
}

#[test]
fn row_with_all_fields_valid() {
    // Explicit test with different valid UUIDs
    let mut obj = serde_json::Map::new();
    obj.insert("item_name".into(), serde_json::json!("Gloves"));
    obj.insert("lot_number".into(), serde_json::json!("LOT-999"));
    obj.insert("quantity_on_hand".into(), serde_json::json!("500"));
    obj.insert("facility_id".into(), serde_json::json!("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11"));
    obj.insert("warehouse_id".into(), serde_json::json!("b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22"));
    obj.insert("bin_id".into(), serde_json::json!("c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33"));
    assert!(validate_import_row_fields(&obj).is_empty());
}
