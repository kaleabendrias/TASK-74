//! Frontend component logic tests.
//!
//! These tests mirror the pure-Rust domain logic extracted from the Yew frontend
//! and verify it without requiring a browser or WASM runtime.  Covered areas:
//!
//! - AuthState reducer (SetAuth / SetUser / Logout transitions)
//! - ToastState reducer (Add / Remove / ordering / deduplication)
//! - Sidebar role-visibility matrix (which sections each role sees)
//! - Route permission matrix (RouteGuard allowed-roles per route)
//! - Form model serialisation/deserialisation (serde round-trips, skip_serializing_if)
//! - PII masking helpers (mask_phone, mask_email)
//! - Client-side validation logic (login fields, deposit cap, period nights, lot qty)

mod test_auth_context;
mod test_toast_context;
mod test_sidebar_visibility;
mod test_route_permissions;
mod test_models_serialization;
mod test_pii_masking;
mod test_client_validation;
mod test_workflow_scenarios;
