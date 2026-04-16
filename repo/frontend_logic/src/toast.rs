//! Pure ToastState reducer — no WASM dependencies.
//!
//! The Yew frontend wraps this in a `Reducible` impl.
//! `frontend_tests` call `reduce` directly.
//!
//! IDs auto-increment starting from 1 (matches production `Default`).

use std::rc::Rc;
use crate::models::{Toast, ToastKind};

/// Maps a `ToastKind` to the CSS class applied by the toast container.
///
/// Mirrors the class expression in `frontend/src/components/toast.rs`:
/// ```ignore
/// let class = match kind {
///     ToastKind::Success => "toast-success",
///     ToastKind::Error   => "toast-error",
///     ToastKind::Info    => "toast-info",
/// };
/// ```
pub fn css_class(kind: &ToastKind) -> &'static str {
    match kind {
        ToastKind::Success => "toast-success",
        ToastKind::Error   => "toast-error",
        ToastKind::Info    => "toast-info",
    }
}

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

impl ToastState {
    /// Pure state transition.
    pub fn reduce(self: Rc<Self>, action: ToastAction) -> Rc<Self> {
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
