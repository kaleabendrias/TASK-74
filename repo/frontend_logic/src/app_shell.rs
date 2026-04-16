//! App-shell and RouteGuard decision logic extracted from Yew components.
//!
//! These pure functions mirror the branching conditions that live inside
//! `frontend/src/components/app.rs` (AppInner) and
//! `frontend/src/components/route_guard.rs` (RouteGuard).
//!
//! Keeping them here lets `frontend_tests` verify component rendering
//! decisions without needing a WASM runtime or a browser.

use crate::models::UserRole;

/// Returns `true` when `AppInner` should render the full authenticated shell
/// (sidebar + `<main class="main-content">` wrapper).
///
/// Mirrors the condition in `frontend/src/components/app.rs`:
/// ```ignore
/// if is_login || !is_authed { /* no shell */ } else { /* shell */ }
/// ```
///
/// * `is_login_route` — the current route is `Route::Login` or `None`.
/// * `is_authed` — `auth.user` is `Some(_)`.
pub fn should_show_shell(is_login_route: bool, is_authed: bool) -> bool {
    !is_login_route && is_authed
}

/// Returns `true` when `RouteGuard` should render its children (grant access).
///
/// Mirrors the condition in `frontend/src/components/route_guard.rs`:
/// ```ignore
/// if props.allowed_roles.contains(&user.role) { render children }
/// else { nav.push(&Route::Forbidden); }
/// ```
///
/// * `user_role` — the authenticated user's role.
/// * `allowed_roles` — the roles listed in the `RouteGuard` `allowed_roles` prop.
pub fn guard_allows(user_role: &UserRole, allowed_roles: &[UserRole]) -> bool {
    allowed_roles.contains(user_role)
}
