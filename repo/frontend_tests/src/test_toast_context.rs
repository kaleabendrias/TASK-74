//! Tests for the ToastState reducer.
//!
//! Imports `frontend_logic::toast` — the shared production reducer — so any
//! change to the production toast logic is caught here immediately.
//!
//! Note: IDs start at 1 (production `Default` has `next_id: 1`).

use std::rc::Rc;
use frontend_logic::toast::{ToastState, ToastAction};
use frontend_logic::models::{Toast, ToastKind};

#[test]
fn initial_state_is_empty() {
    let s = ToastState::default();
    assert!(s.toasts.is_empty());
    assert_eq!(s.next_id, 1);
}

#[test]
fn add_creates_toast_with_id_1() {
    let state = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Info, "hello".into()));
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.toasts[0].id, 1);
    assert_eq!(state.toasts[0].message, "hello");
    assert_eq!(state.toasts[0].kind, ToastKind::Info);
}

#[test]
fn next_id_increments_after_each_add() {
    let s = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Info, "a".into()));
    assert_eq!(s.next_id, 2);
    let s2 = s.reduce(ToastAction::Add(ToastKind::Success, "b".into()));
    assert_eq!(s2.next_id, 3);
}

#[test]
fn multiple_toasts_accumulate_in_order() {
    let s = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Info, "first".into()))
        .reduce(ToastAction::Add(ToastKind::Error, "second".into()))
        .reduce(ToastAction::Add(ToastKind::Success, "third".into()));
    assert_eq!(s.toasts.len(), 3);
    assert_eq!(s.toasts[0].message, "first");
    assert_eq!(s.toasts[2].message, "third");
}

#[test]
fn remove_eliminates_correct_toast_by_id() {
    let s = Rc::new(ToastState {
        toasts: vec![
            Toast { id: 1, kind: ToastKind::Info,    message: "first".into() },
            Toast { id: 2, kind: ToastKind::Success, message: "second".into() },
            Toast { id: 3, kind: ToastKind::Error,   message: "third".into() },
        ],
        next_id: 4,
    });
    let s2 = s.reduce(ToastAction::Remove(2));
    assert_eq!(s2.toasts.len(), 2);
    assert!(s2.toasts.iter().all(|t| t.id != 2));
    assert_eq!(s2.toasts[0].id, 1);
    assert_eq!(s2.toasts[1].id, 3);
}

#[test]
fn remove_non_existent_id_is_safe() {
    let s = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Info, "only".into()));
    let s2 = s.reduce(ToastAction::Remove(999));
    assert_eq!(s2.toasts.len(), 1);
}

#[test]
fn remove_all_toasts_one_by_one() {
    let s = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Info, "a".into()))
        .reduce(ToastAction::Add(ToastKind::Info, "b".into()))
        .reduce(ToastAction::Add(ToastKind::Info, "c".into()));
    assert_eq!(s.toasts.len(), 3);
    let ids: Vec<u32> = s.toasts.iter().map(|t| t.id).collect();
    let s = s.reduce(ToastAction::Remove(ids[0]));
    let s = s.reduce(ToastAction::Remove(ids[1]));
    let s = s.reduce(ToastAction::Remove(ids[2]));
    assert!(s.toasts.is_empty());
}

#[test]
fn remove_from_empty_state_is_safe() {
    let s = Rc::new(ToastState::default()).reduce(ToastAction::Remove(42));
    assert!(s.toasts.is_empty());
}

#[test]
fn error_kind_stored_correctly() {
    let s = Rc::new(ToastState::default())
        .reduce(ToastAction::Add(ToastKind::Error, "oops".into()));
    assert_eq!(s.toasts[0].kind, ToastKind::Error);
}

#[test]
fn ids_are_unique_across_adds() {
    let mut s = Rc::new(ToastState::default());
    for i in 0..10 {
        s = s.reduce(ToastAction::Add(ToastKind::Info, format!("msg {}", i)));
    }
    let ids: Vec<u32> = s.toasts.iter().map(|t| t.id).collect();
    let unique: std::collections::HashSet<u32> = ids.iter().cloned().collect();
    assert_eq!(ids.len(), unique.len(), "All toast IDs must be unique");
}
