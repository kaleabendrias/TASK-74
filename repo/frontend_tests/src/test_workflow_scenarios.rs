//! Multi-step workflow scenario tests.
//!
//! Each test simulates a realistic sequence of state transitions that a real
//! user session would exercise — auth change, visibility gate, form validation,
//! and toast notification — rather than testing each reducer in isolation.

use std::rc::Rc;

use frontend_logic::auth::{AuthState, AuthAction};
use frontend_logic::toast::{ToastState, ToastAction};
use frontend_logic::models::{UserProfile, UserRole, ToastKind};
use frontend_logic::sidebar::visible_sections;
use frontend_logic::routing::{Route, can_access};
use frontend_logic::validation::{validate_login, validate_deposit_cap};

// ── Tests ─────────────────────────────────────────────────────────────────────

/// A publisher logs in, navigates to a permitted route, then the session expires
/// (Logout). Verifies state is clean and route access is restricted afterward.
#[test]
fn publisher_session_lifecycle() {
    let auth = Rc::new(AuthState::default())
        .reduce(AuthAction::SetAuth {
            user: UserProfile {
                id: "".into(),
                username: "publisher".into(),
                role: UserRole::Publisher,
                facility_id: None,
                mfa_enabled: false,
                created_at: "".into(),
            },
            csrf_token: "csrf-abc".into(),
        });

    // Publisher can access ResourceNew but not Inventory
    let role = &auth.user.as_ref().unwrap().role;
    assert!(can_access(role, &Route::ResourceNew));
    assert!(!can_access(role, &Route::Inventory));

    // Publisher sees Content + Account in sidebar, not System
    let sections = visible_sections(role);
    assert!(sections.contains(&"Content"));
    assert!(!sections.contains(&"System"));
    assert!(!sections.contains(&"Inventory"));

    // Session ends
    let auth = auth.reduce(AuthAction::Logout);
    assert!(auth.user.is_none());
    assert!(auth.csrf_token.is_none());
}

/// Reviewer approves a resource: login, check visibility, check route, emit toast.
#[test]
fn reviewer_approves_resource_emits_success_toast() {
    let auth = Rc::new(AuthState::default())
        .reduce(AuthAction::SetAuth {
            user: UserProfile {
                id: "".into(),
                username: "reviewer".into(),
                role: UserRole::Reviewer,
                facility_id: None,
                mfa_enabled: false,
                created_at: "".into(),
            },
            csrf_token: "csrf-rev".into(),
        });

    let role = &auth.user.as_ref().unwrap().role;
    // Reviewer can navigate to resource list
    assert!(can_access(role, &Route::ResourceList));
    // Reviewer cannot go to configuration
    assert!(!can_access(role, &Route::Configuration));

    // Simulate emitting a toast after approval
    let toasts = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Success, "Resource approved".into()));
    assert_eq!(toasts.toasts.len(), 1);
    assert_eq!(toasts.toasts[0].kind, ToastKind::Success);
    assert!(toasts.toasts[0].message.contains("approved"));
}

/// Inventory clerk tries a forbidden route: toast fires, redirect enforced.
#[test]
fn inventory_clerk_blocked_from_content_routes() {
    let auth = Rc::new(AuthState::default())
        .reduce(AuthAction::SetAuth {
            user: UserProfile {
                id: "".into(),
                username: "clerk".into(),
                role: UserRole::InventoryClerk,
                facility_id: Some("fac-1".into()),
                mfa_enabled: false,
                created_at: "".into(),
            },
            csrf_token: "csrf-clk".into(),
        });

    let role = &auth.user.as_ref().unwrap().role;
    assert!(!can_access(role, &Route::ResourceNew));
    assert!(!can_access(role, &Route::Configuration));
    assert!(!can_access(role, &Route::LodgingList));
    assert!(can_access(role, &Route::Inventory));

    // Simulate "forbidden" toast emitted by the route guard
    let toasts = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Error, "Access denied".into()));
    assert_eq!(toasts.toasts[0].kind, ToastKind::Error);
}

/// Login form validation: empty username or short password blocks submit.
#[test]
fn login_form_prevents_submit_on_invalid_input() {
    // Both fields empty
    let e = validate_login("", "");
    assert!(e.iter().any(|(f, _)| *f == "username"));
    assert!(e.iter().any(|(f, _)| *f == "password"));

    // Username present but password too short (< 4 chars)
    let e = validate_login("admin", "abc");
    assert!(!e.iter().any(|(f, _)| *f == "username"));
    assert!(e.iter().any(|(_, m)| m.contains("4 characters")));

    // Valid credentials
    assert!(validate_login("admin", "Admin@2024").is_empty());
}

/// Deposit cap validation across the lodging creation workflow.
/// Simulates user entering rent, changing deposit, seeing/clearing warning.
#[test]
fn lodging_form_deposit_cap_workflow() {
    // User types rent = 1000
    let rent = 1000.0_f64;

    // Types deposit = 1200 (OK)
    assert!(validate_deposit_cap(1200.0, rent));
    // Types deposit = 1500 (at the boundary — still OK)
    assert!(validate_deposit_cap(1500.0, rent));
    // Types deposit = 1501 (over cap — warning)
    assert!(!validate_deposit_cap(1501.0, rent));
    // Corrects to 1400 — warning clears
    assert!(validate_deposit_cap(1400.0, rent));
}

/// MFA-enabled user session: token and MFA flag survive a SetAuth.
#[test]
fn mfa_user_session_is_intact_after_set_auth() {
    let auth = Rc::new(AuthState::default())
        .reduce(AuthAction::SetAuth {
            user: UserProfile {
                id: "".into(),
                username: "admin".into(),
                role: UserRole::Administrator,
                facility_id: None,
                mfa_enabled: true,
                created_at: "".into(),
            },
            csrf_token: "mfa-tok".into(),
        });

    let profile = auth.user.as_ref().unwrap();
    assert!(profile.mfa_enabled);
    assert_eq!(auth.csrf_token.as_deref(), Some("mfa-tok"));
    assert_eq!(profile.username, "admin");
}

/// Multiple toasts queued and dismissed in order.
#[test]
fn toast_queue_lifecycle_over_multiple_events() {
    let toasts = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Info, "Import started".into()))
        .reduce(ToastAction::Add(ToastKind::Info, "Row 50 processed".into()))
        .reduce(ToastAction::Add(ToastKind::Success, "Import complete".into()));

    assert_eq!(toasts.toasts.len(), 3);
    assert_eq!(toasts.toasts[0].id, 1);
    assert_eq!(toasts.toasts[2].kind, ToastKind::Success);

    // User dismisses the first toast (id=1)
    let toasts = toasts.reduce(ToastAction::Remove(1));
    assert_eq!(toasts.toasts.len(), 2);
    assert_eq!(toasts.toasts[0].id, 2); // second toast is now first

    // Dismiss remaining
    let toasts = toasts
        .reduce(ToastAction::Remove(2))
        .reduce(ToastAction::Remove(3));
    assert!(toasts.toasts.is_empty());
}

/// Administrator has unrestricted route access across all portal sections.
#[test]
fn administrator_can_access_all_routes() {
    let role = UserRole::Administrator;
    for route in &[
        Route::ResourceNew, Route::Inventory, Route::Configuration,
        Route::SecuritySettings, Route::LodgingList, Route::Dashboard,
    ] {
        assert!(
            can_access(&role, route),
            "Administrator must be able to access {:?}",
            route
        );
    }
    let sections = visible_sections(&role);
    assert!(sections.contains(&"Content"));
    assert!(sections.contains(&"Inventory"));
    assert!(sections.contains(&"System"));
    assert!(sections.contains(&"Data"));
    assert!(sections.contains(&"Account"));
}

/// Clinician scope: can view lodgings and inventory but not publish resources.
#[test]
fn clinician_can_view_lodgings_and_inventory_but_not_publish() {
    let role = UserRole::Clinician;
    assert!(can_access(&role, &Route::Inventory));
    assert!(can_access(&role, &Route::LodgingList));
    assert!(!can_access(&role, &Route::ResourceNew));
    assert!(!can_access(&role, &Route::Configuration));

    let sections = visible_sections(&role);
    assert!(sections.contains(&"Inventory"));
    assert!(sections.contains(&"Content"));
    assert!(!sections.contains(&"System"));
}

/// Verifies that toast IDs auto-increment across multiple Add actions.
#[test]
fn toast_ids_are_monotonically_increasing() {
    let mut state = Rc::new(ToastState::default());
    for i in 0..5_u32 {
        state = state.reduce(ToastAction::Add(ToastKind::Info, format!("msg {}", i)));
        assert_eq!(state.toasts.last().unwrap().id, i + 1);
    }
    assert_eq!(state.next_id, 6);
}
