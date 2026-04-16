//! Shared pure-Rust domain logic for the Tourism Portal frontend.
//!
//! This crate contains every piece of logic that the Yew frontend uses but that
//! can be exercised without a browser or WASM runtime:
//!
//! - **models**: All API request/response DTOs (serde round-trips, skip_serializing_if).
//! - **auth**: `AuthState` / `AuthAction` + pure `reduce` (no CSRF side-effect).
//! - **toast**: `ToastState` / `ToastAction` + pure `reduce`.
//! - **sidebar**: `visible_sections(role) -> Vec<&str>` — role-conditional section logic.
//! - **routing**: `Route` enum + `can_access(role, route)` — route-guard permission matrix.
//! - **validation**: Client-side field validators (login, deposit cap, period nights, etc.).
//! - **mask**: PII masking helpers (`mask_phone`, `mask_email`).

pub mod models;
pub mod auth;
pub mod toast;
pub mod sidebar;
pub mod routing;
pub mod validation;
pub mod mask;
pub mod app_shell;
