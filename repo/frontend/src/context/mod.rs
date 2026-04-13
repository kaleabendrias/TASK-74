use std::rc::Rc;
use yew::prelude::*;

use crate::models::{UserProfile, UserRole, Toast, ToastKind};
use crate::services::api;

// ── Auth Context ──
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
    Logout,
}

impl Reducible for AuthState {
    type Action = AuthAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AuthAction::SetAuth { user, csrf_token } => {
                api::set_csrf_token(Some(csrf_token.clone()));
                Rc::new(AuthState {
                    user: Some(user),
                    csrf_token: Some(csrf_token),
                })
            }
            AuthAction::Logout => {
                api::set_csrf_token(None);
                Rc::new(AuthState::default())
            }
        }
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

// ── Toast Context ──
#[derive(Debug, Clone, PartialEq)]
pub struct ToastState {
    pub toasts: Vec<Toast>,
    pub next_id: u32,
}

impl Default for ToastState {
    fn default() -> Self {
        Self { toasts: vec![], next_id: 1 }
    }
}

pub enum ToastAction {
    Add(ToastKind, String),
    Remove(u32),
}

impl Reducible for ToastState {
    type Action = ToastAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut toasts = self.toasts.clone();
        let mut next_id = self.next_id;
        match action {
            ToastAction::Add(kind, message) => {
                toasts.push(Toast { id: next_id, kind, message });
                next_id += 1;
            }
            ToastAction::Remove(id) => {
                toasts.retain(|t| t.id != id);
            }
        }
        Rc::new(ToastState { toasts, next_id })
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
