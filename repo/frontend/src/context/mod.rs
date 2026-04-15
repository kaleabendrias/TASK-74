//! Application-wide contexts: AuthProvider (user profile + CSRF token in memory)
//! and ToastProvider (notification queue with auto-dismiss).
//!
//! Pure state-transition logic lives in `frontend_logic::{auth,toast}`.
//! This module wraps those types in Yew's `Reducible` trait and handles
//! the WASM-specific side effect: syncing the CSRF token to a thread-local
//! so that `services::api` can inject it into every request.

use std::rc::Rc;
use yew::prelude::*;

use crate::services::api;

// ── Re-export action/state types from shared logic ────────────────────────────

pub use frontend_logic::auth::AuthAction;
pub use frontend_logic::toast::ToastAction;
pub use frontend_logic::models::{UserProfile, UserRole, Toast, ToastKind};

// ── Auth Context ──────────────────────────────────────────────────────────────

/// Newtype wrapper so the frontend can implement the Yew `Reducible` trait
/// (which lives in an external crate) on a locally-owned type.
/// All field access is forwarded via `Deref` to the inner `frontend_logic` type.
pub struct AuthState(pub frontend_logic::auth::AuthState);

impl Default for AuthState {
    fn default() -> Self {
        Self(frontend_logic::auth::AuthState::default())
    }
}

impl std::ops::Deref for AuthState {
    type Target = frontend_logic::auth::AuthState;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Reducible for AuthState {
    type Action = AuthAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        // Side-effect: keep CSRF token in thread-local for api::* functions.
        match &action {
            AuthAction::SetAuth { csrf_token, .. } =>
                api::set_csrf_token(Some(csrf_token.clone())),
            AuthAction::Logout => api::set_csrf_token(None),
            _ => {}
        }
        // Pure state transition delegated to frontend_logic.
        let inner = Rc::new(self.0.clone());
        let new_inner = inner.reduce(action);
        Rc::new(AuthState((*new_inner).clone()))
    }
}

pub type AuthContext = UseReducerHandle<AuthState>;

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    pub children: Children,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let auth = use_reducer(AuthState::default);
    html! {
        <ContextProvider<AuthContext> context={auth}>
            { props.children.clone() }
        </ContextProvider<AuthContext>>
    }
}

// ── Toast Context ─────────────────────────────────────────────────────────────

/// Newtype wrapper for `Reducible` — same pattern as `AuthState`.
pub struct ToastState(pub frontend_logic::toast::ToastState);

impl Default for ToastState {
    fn default() -> Self {
        Self(frontend_logic::toast::ToastState::default())
    }
}

impl std::ops::Deref for ToastState {
    type Target = frontend_logic::toast::ToastState;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Reducible for ToastState {
    type Action = ToastAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let inner = Rc::new(self.0.clone());
        let new_inner = inner.reduce(action);
        Rc::new(ToastState((*new_inner).clone()))
    }
}

pub type ToastContext = UseReducerHandle<ToastState>;

#[derive(Properties, PartialEq)]
pub struct ToastProviderProps {
    pub children: Children,
}

#[function_component(ToastProvider)]
pub fn toast_provider(props: &ToastProviderProps) -> Html {
    let state = use_reducer(ToastState::default);
    html! {
        <ContextProvider<ToastContext> context={state}>
            { props.children.clone() }
        </ContextProvider<ToastContext>>
    }
}
