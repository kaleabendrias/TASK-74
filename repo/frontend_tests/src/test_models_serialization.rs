//! Serde round-trip tests for shared request/response models.
//!
//! Imports from `frontend_logic::models` — the same definitions used by the
//! frontend — so any serde annotation change is caught here.

use serde_json::json;
use frontend_logic::models::{
    LoginRequest, UserProfile, UserRole,
    CreateResourceRequest, UpdateResourceRequest,
    RentChangeRequest, CounterproposalRequest,
    CreateLodgingRequest, ExportRequestBody,
};

#[test]
fn login_request_serializes_totp_as_null_when_none() {
    let req = LoginRequest { username: "admin".into(), password: "pass".into(), totp_code: None };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["totp_code"], serde_json::Value::Null);
}

#[test]
fn login_request_serializes_totp_code_when_present() {
    let req = LoginRequest {
        username: "admin".into(), password: "pw".into(),
        totp_code: Some("123456".into()),
    };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["totp_code"].as_str().unwrap(), "123456");
}

#[test]
fn create_resource_omits_none_optional_fields() {
    let req = CreateResourceRequest {
        title: "Park".into(), address: "1 Main".into(),
        category: None, tags: vec![], hours: json!({}), pricing: json!({}),
        latitude: None, longitude: None, media_refs: vec![],
        scheduled_publish_at: None,
        tz_offset_minutes: None, contact_info: None,
    };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert!(v.get("tz_offset_minutes").is_none(), "None should be absent");
    assert!(v.get("contact_info").is_none(), "None should be absent");
}

#[test]
fn create_resource_includes_present_optional_fields() {
    let req = CreateResourceRequest {
        title: "Park".into(), address: "1 Main".into(),
        category: None, tags: vec![], hours: json!({}), pricing: json!({}),
        latitude: None, longitude: None, media_refs: vec![],
        scheduled_publish_at: None,
        tz_offset_minutes: Some(-300), contact_info: Some("email@test.com".into()),
    };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["tz_offset_minutes"], -300);
    assert_eq!(v["contact_info"], "email@test.com");
}

#[test]
fn update_resource_with_all_none_produces_empty_object() {
    let req = UpdateResourceRequest {
        title: None, category: None, tags: None, hours: None,
        pricing: None, address: None, latitude: None, longitude: None,
        media_refs: None, state: None, scheduled_publish_at: None,
        tz_offset_minutes: None, contact_info: None,
    };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v, json!({}));
}

#[test]
fn update_resource_with_only_title_includes_only_title() {
    let req = UpdateResourceRequest {
        title: Some("New Title".into()),
        category: None, tags: None, hours: None, pricing: None,
        address: None, latitude: None, longitude: None, media_refs: None,
        state: None, scheduled_publish_at: None,
        tz_offset_minutes: None, contact_info: None,
    };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["title"], "New Title");
    assert!(v.get("state").is_none());
    assert!(v.get("tags").is_none());
}

#[test]
fn update_resource_state_transition_serialises() {
    let req = UpdateResourceRequest {
        state: Some("in_review".into()),
        title: None, category: None, tags: None, hours: None, pricing: None,
        address: None, latitude: None, longitude: None, media_refs: None,
        scheduled_publish_at: None, tz_offset_minutes: None, contact_info: None,
    };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["state"], "in_review");
}

#[test]
fn user_profile_round_trips_through_json() {
    let json_str = r#"{
        "id": "uuid-1",
        "username": "admin",
        "role": "Administrator",
        "facility_id": null,
        "mfa_enabled": false,
        "created_at": "2024-01-01T00:00:00Z"
    }"#;
    let p: UserProfile = serde_json::from_str(json_str).unwrap();
    assert_eq!(p.username, "admin");
    assert_eq!(p.role, UserRole::Administrator);
    assert!(p.facility_id.is_none());
}

#[test]
fn user_profile_with_facility_deserialises() {
    let json_str = r#"{
        "id": "uuid-2",
        "username": "clerk",
        "role": "InventoryClerk",
        "facility_id": "fac-1",
        "mfa_enabled": true,
        "created_at": "2024-01-01T00:00:00Z"
    }"#;
    let p: UserProfile = serde_json::from_str(json_str).unwrap();
    assert_eq!(p.facility_id.as_deref(), Some("fac-1"));
    assert!(p.mfa_enabled);
}

#[test]
fn rent_change_request_serialises_correctly() {
    let req = RentChangeRequest { proposed_rent: 2200.0, proposed_deposit: 2200.0 };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["proposed_rent"], 2200.0);
    assert_eq!(v["proposed_deposit"], 2200.0);
}

#[test]
fn counterproposal_request_serialises_correctly() {
    let req = CounterproposalRequest { proposed_rent: 2100.0, proposed_deposit: 2000.0 };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["proposed_rent"], 2100.0);
    assert_eq!(v["proposed_deposit"], 2000.0);
}

#[test]
fn create_lodging_omits_none_description() {
    let req = CreateLodgingRequest {
        name: "Suite".into(), description: None,
        amenities: vec!["wifi".into()],
        facility_id: None,
        monthly_rent: Some(1500.0), deposit_amount: Some(1500.0),
    };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert!(v.get("description").is_none());
}

#[test]
fn export_request_body_serialises() {
    let req = ExportRequestBody { export_type: "resources".into() };
    let v: serde_json::Value = serde_json::to_value(&req).unwrap();
    assert_eq!(v["export_type"], "resources");
}

#[test]
fn all_user_roles_deserialise_from_json_strings() {
    let cases = [
        ("\"Administrator\"",  UserRole::Administrator),
        ("\"Publisher\"",      UserRole::Publisher),
        ("\"Reviewer\"",       UserRole::Reviewer),
        ("\"Clinician\"",      UserRole::Clinician),
        ("\"InventoryClerk\"", UserRole::InventoryClerk),
    ];
    for (json, expected) in cases {
        let role: UserRole = serde_json::from_str(json).unwrap();
        assert_eq!(role, expected);
    }
}
