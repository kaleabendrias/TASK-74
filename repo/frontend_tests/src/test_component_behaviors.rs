//! Unit-level coverage for the logic that drives Yew component rendering.
//!
//! The actual components live in `frontend/src/components/` and require a WASM
//! runtime.  This module exercises the pure-Rust functions extracted into
//! `frontend_logic` that power each component's branching decisions:
//!
//! | Component            | Logic tested here                                |
//! |----------------------|--------------------------------------------------|
//! | `app.rs` (AppInner)  | `app_shell::should_show_shell`                   |
//! | `route_guard.rs`     | `app_shell::guard_allows`                        |
//! | `toast.rs`           | `toast::css_class`                               |
//! | `sidebar.rs`         | `sidebar::avatar_initial`, `sidebar::visible_sections` (lifecycle) |

use frontend_logic::{
    app_shell::{guard_allows, should_show_shell},
    models::{Toast, ToastKind, UserRole},
    sidebar::{avatar_initial, visible_sections},
    toast::{css_class, ToastAction, ToastState},
};
use std::rc::Rc;

// ── AppInner shell layout decision ────────────────────────────────────────────

#[test]
fn shell_shown_when_authed_and_not_login_route() {
    assert!(should_show_shell(false, true),
        "authenticated user on a non-login route must see the shell");
}

#[test]
fn shell_hidden_on_login_route_even_when_authed() {
    assert!(!should_show_shell(true, true),
        "login route never shows the shell, even if a session exists");
}

#[test]
fn shell_hidden_when_not_authed_on_non_login_route() {
    assert!(!should_show_shell(false, false),
        "unauthenticated users must not see the authenticated shell");
}

#[test]
fn shell_hidden_when_not_authed_on_login_route() {
    // The double-false edge case: unauthenticated visitor on the login page.
    assert!(!should_show_shell(true, false));
}

#[test]
fn shell_logic_is_exclusive_to_authed_non_login_routes() {
    // Enumerate all four combinations; only (false, true) produces true.
    let cases = [
        (false, false, false),
        (false, true,  true),
        (true,  false, false),
        (true,  true,  false),
    ];
    for (is_login, is_authed, expected) in cases {
        assert_eq!(
            should_show_shell(is_login, is_authed),
            expected,
            "is_login={is_login} is_authed={is_authed}"
        );
    }
}

// ── RouteGuard access decision ─────────────────────────────────────────────────

#[test]
fn guard_allows_when_role_is_in_allowed_list() {
    assert!(guard_allows(
        &UserRole::Administrator,
        &[UserRole::Administrator],
    ));
}

#[test]
fn guard_allows_when_role_appears_among_several_allowed_roles() {
    assert!(guard_allows(
        &UserRole::Publisher,
        &[UserRole::Administrator, UserRole::Publisher],
    ));
}

#[test]
fn guard_denies_when_role_not_in_allowed_list() {
    assert!(!guard_allows(
        &UserRole::Reviewer,
        &[UserRole::Administrator],
    ));
}

#[test]
fn guard_denies_when_allowed_list_is_empty() {
    assert!(!guard_allows(&UserRole::Administrator, &[]));
}

#[test]
fn guard_configuration_route_allows_only_admin() {
    // Matches the RouteGuard in configuration.rs:
    // allowed_roles={vec![UserRole::Administrator]}
    let config_allowed = [UserRole::Administrator];
    let all_roles = [
        UserRole::Administrator,
        UserRole::Publisher,
        UserRole::Reviewer,
        UserRole::Clinician,
        UserRole::InventoryClerk,
    ];
    for role in &all_roles {
        let expected = matches!(role, UserRole::Administrator);
        assert_eq!(
            guard_allows(role, &config_allowed),
            expected,
            "{role:?} vs configuration route"
        );
    }
}

#[test]
fn guard_all_roles_allowed_when_list_contains_every_role() {
    let all_allowed = [
        UserRole::Administrator,
        UserRole::Publisher,
        UserRole::Reviewer,
        UserRole::Clinician,
        UserRole::InventoryClerk,
    ];
    for role in &all_allowed {
        assert!(guard_allows(role, &all_allowed), "{role:?} should be allowed");
    }
}

// ── Toast CSS class mapping ────────────────────────────────────────────────────

#[test]
fn toast_success_maps_to_toast_success_class() {
    assert_eq!(css_class(&ToastKind::Success), "toast-success");
}

#[test]
fn toast_error_maps_to_toast_error_class() {
    assert_eq!(css_class(&ToastKind::Error), "toast-error");
}

#[test]
fn toast_info_maps_to_toast_info_class() {
    assert_eq!(css_class(&ToastKind::Info), "toast-info");
}

#[test]
fn toast_classes_are_distinct() {
    let success = css_class(&ToastKind::Success);
    let error   = css_class(&ToastKind::Error);
    let info    = css_class(&ToastKind::Info);
    assert_ne!(success, error);
    assert_ne!(success, info);
    assert_ne!(error,   info);
}

#[test]
fn toast_css_class_each_kind_has_toast_prefix() {
    for kind in [ToastKind::Success, ToastKind::Error, ToastKind::Info] {
        let cls = css_class(&kind);
        assert!(cls.starts_with("toast-"), "class '{cls}' must start with 'toast-'");
    }
}

// Toast state: verify that the kind stored in ToastState matches what css_class
// would render (coupling the state reducer to the CSS mapping).
#[test]
fn toast_state_stores_kind_that_css_class_accepts() {
    let state = Rc::new(ToastState::default());
    let state = state.reduce(ToastAction::Add(ToastKind::Success, "saved".into()));
    let state = state.reduce(ToastAction::Add(ToastKind::Error,   "oops".into()));
    let state = state.reduce(ToastAction::Add(ToastKind::Info,    "note".into()));

    assert_eq!(state.toasts.len(), 3);
    assert_eq!(css_class(&state.toasts[0].kind), "toast-success");
    assert_eq!(css_class(&state.toasts[1].kind), "toast-error");
    assert_eq!(css_class(&state.toasts[2].kind), "toast-info");
}

// ── Sidebar avatar initial ────────────────────────────────────────────────────

#[test]
fn avatar_initial_returns_uppercased_first_char() {
    assert_eq!(avatar_initial("admin"),   "A");
    assert_eq!(avatar_initial("publisher"), "P");
    assert_eq!(avatar_initial("reviewer"), "R");
    assert_eq!(avatar_initial("clinician"), "C");
    assert_eq!(avatar_initial("clerk"),   "C");
}

#[test]
fn avatar_initial_already_uppercase_stays_uppercase() {
    assert_eq!(avatar_initial("Alice"), "A");
    assert_eq!(avatar_initial("Bob"),   "B");
}

#[test]
fn avatar_initial_lowercase_input_is_uppercased() {
    assert_eq!(avatar_initial("zara"), "Z");
}

#[test]
fn avatar_initial_empty_username_returns_question_mark() {
    assert_eq!(avatar_initial(""), "?");
}

#[test]
fn avatar_initial_single_char_username() {
    assert_eq!(avatar_initial("x"), "X");
}

#[test]
fn avatar_initial_unicode_username() {
    // Non-ASCII usernames: the initial is the first Unicode scalar value.
    let init = avatar_initial("über");
    assert!(!init.is_empty(), "initial must not be empty for unicode usernames");
}

// ── Sidebar visible_sections lifecycle coupling ────────────────────────────────
// These tests extend the existing coverage by verifying section membership from
// a component-intent perspective (what does the sidebar *render* for each role).

#[test]
fn admin_sidebar_renders_all_five_non_main_sections() {
    let sections = visible_sections(&UserRole::Administrator);
    for s in ["Content", "Inventory", "Data", "System", "Account"] {
        assert!(sections.contains(&s),
            "Admin sidebar must contain '{s}'");
    }
}

#[test]
fn inventory_clerk_sidebar_has_no_system_section() {
    let sections = visible_sections(&UserRole::InventoryClerk);
    assert!(!sections.contains(&"System"),
        "InventoryClerk must not see the System (Configuration) section");
}

#[test]
fn publisher_sidebar_has_no_inventory_section() {
    let sections = visible_sections(&UserRole::Publisher);
    assert!(!sections.contains(&"Inventory"),
        "Publisher must not see the Inventory section");
}

#[test]
fn reviewer_sidebar_has_no_system_section() {
    let sections = visible_sections(&UserRole::Reviewer);
    assert!(!sections.contains(&"System"),
        "Reviewer must not see the System section");
}

#[test]
fn clinician_sidebar_has_no_system_or_data_section() {
    let sections = visible_sections(&UserRole::Clinician);
    assert!(!sections.contains(&"System"),
        "Clinician must not see System");
    assert!(!sections.contains(&"Data"),
        "Clinician must not see Data");
}
