//! Tests for the AuthState reducer.
//!
//! Imports `frontend_logic::auth` directly — the same code the Yew frontend
//! uses in `context/mod.rs` — so any change to the production reducer is
//! immediately tested here.

use std::rc::Rc;
use frontend_logic::auth::{AuthState, AuthAction};
use frontend_logic::models::{UserProfile, UserRole};

fn profile(username: &str, role: UserRole) -> UserProfile {
    UserProfile {
        id: "id-1".into(),
        username: username.into(),
        role,
        facility_id: None,
        mfa_enabled: false,
        created_at: "".into(),
    }
}

#[test]
fn initial_state_is_unauthenticated() {
    let s = AuthState::default();
    assert!(s.user.is_none());
    assert!(s.csrf_token.is_none());
}

#[test]
fn set_auth_sets_user_and_token() {
    let s = Rc::new(AuthState::default());
    let s2 = s.reduce(AuthAction::SetAuth {
        user: profile("admin", UserRole::Administrator),
        csrf_token: "tok-1".into(),
    });
    assert_eq!(s2.user.as_ref().unwrap().username, "admin");
    assert_eq!(s2.csrf_token.as_deref(), Some("tok-1"));
}

#[test]
fn set_user_replaces_user_but_preserves_token() {
    let s = Rc::new(AuthState {
        user: Some(profile("admin", UserRole::Administrator)),
        csrf_token: Some("existing".into()),
    });
    let s2 = s.reduce(AuthAction::SetUser(profile("publisher", UserRole::Publisher)));
    assert_eq!(s2.user.as_ref().unwrap().username, "publisher");
    assert_eq!(s2.csrf_token.as_deref(), Some("existing"));
}

#[test]
fn set_user_on_tokenless_state_keeps_no_token() {
    let s = Rc::new(AuthState::default());
    let s2 = s.reduce(AuthAction::SetUser(profile("reviewer", UserRole::Reviewer)));
    assert_eq!(s2.user.as_ref().unwrap().username, "reviewer");
    assert!(s2.csrf_token.is_none());
}

#[test]
fn logout_clears_user_and_token() {
    let s = Rc::new(AuthState {
        user: Some(profile("clerk", UserRole::InventoryClerk)),
        csrf_token: Some("tok-x".into()),
    });
    let s2 = s.reduce(AuthAction::Logout);
    assert!(s2.user.is_none());
    assert!(s2.csrf_token.is_none());
}

#[test]
fn set_auth_then_logout_returns_to_default() {
    let s = Rc::new(AuthState::default())
        .reduce(AuthAction::SetAuth {
            user: profile("clinician", UserRole::Clinician),
            csrf_token: "tok".into(),
        });
    assert!(s.user.is_some());
    let s2 = s.reduce(AuthAction::Logout);
    assert!(s2.user.is_none());
    assert!(s2.csrf_token.is_none());
}

#[test]
fn set_auth_twice_overrides_previous_session() {
    let s = Rc::new(AuthState::default())
        .reduce(AuthAction::SetAuth {
            user: profile("admin", UserRole::Administrator),
            csrf_token: "first".into(),
        });
    let s2 = s.reduce(AuthAction::SetAuth {
        user: profile("reviewer", UserRole::Reviewer),
        csrf_token: "second".into(),
    });
    assert_eq!(s2.user.as_ref().unwrap().username, "reviewer");
    assert_eq!(s2.csrf_token.as_deref(), Some("second"));
}

#[test]
fn set_auth_works_for_all_roles() {
    for (name, role) in [
        ("admin",     UserRole::Administrator),
        ("publisher", UserRole::Publisher),
        ("reviewer",  UserRole::Reviewer),
        ("clinician", UserRole::Clinician),
        ("clerk",     UserRole::InventoryClerk),
    ] {
        let s = Rc::new(AuthState::default()).reduce(AuthAction::SetAuth {
            user: profile(name, role),
            csrf_token: "t".into(),
        });
        assert_eq!(s.user.as_ref().unwrap().username, name);
    }
}

#[test]
fn mfa_enabled_flag_preserved_through_set_auth() {
    let mut p = profile("admin", UserRole::Administrator);
    p.mfa_enabled = true;
    let s = Rc::new(AuthState::default()).reduce(AuthAction::SetAuth {
        user: p,
        csrf_token: "t".into(),
    });
    assert!(s.user.as_ref().unwrap().mfa_enabled);
}

#[test]
fn facility_scoped_user_sets_facility_id() {
    let mut p = profile("clerk", UserRole::InventoryClerk);
    p.facility_id = Some("fac-1".into());
    let s = Rc::new(AuthState::default()).reduce(AuthAction::SetAuth {
        user: p,
        csrf_token: "t".into(),
    });
    assert_eq!(s.user.as_ref().unwrap().facility_id.as_deref(), Some("fac-1"));
}
