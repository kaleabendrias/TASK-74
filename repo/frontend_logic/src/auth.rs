//! Pure AuthState reducer — no WASM, no side-effects.
//!
//! The Yew frontend wraps this in a `Reducible` impl that also calls
//! `api::set_csrf_token()`.  The `frontend_tests` crate calls `reduce`
//! directly to test the state-transition logic in isolation.

use std::rc::Rc;
use crate::models::{UserProfile};

#[derive(Debug, Clone, PartialEq)]
pub struct AuthState {
    pub user: Option<UserProfile>,
    pub csrf_token: Option<String>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self { user: None, csrf_token: None }
    }
}

pub enum AuthAction {
    SetAuth { user: UserProfile, csrf_token: String },
    SetUser(UserProfile),
    Logout,
}

impl AuthState {
    /// Pure state transition: no side-effects.
    /// The CSRF token side-effect (`api::set_csrf_token`) is handled by
    /// the frontend's `Reducible` wrapper in `context/mod.rs`.
    pub fn reduce(self: Rc<Self>, action: AuthAction) -> Rc<Self> {
        match action {
            AuthAction::SetAuth { user, csrf_token } =>
                Rc::new(AuthState { user: Some(user), csrf_token: Some(csrf_token) }),
            AuthAction::SetUser(user) =>
                Rc::new(AuthState { user: Some(user), csrf_token: self.csrf_token.clone() }),
            AuthAction::Logout =>
                Rc::new(AuthState::default()),
        }
    }
}
