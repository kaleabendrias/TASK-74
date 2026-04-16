//! Globally mounted toast notification container.
//! Renders success/error/info toasts with 4-second auto-dismiss.
//!
//! Each toast is rendered by a dedicated `ToastItem` function_component so that
//! `use_effect_with` is called at the component level (not inside a loop), which
//! is required by Yew's hook rules and ensures the auto-dismiss timer is reliably
//! created and fired exactly once per toast.

use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::context::{ToastAction, ToastContext};
use crate::models::{Toast, ToastKind};

// ── Per-toast item ────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct ToastItemProps {
    toast: Toast,
    on_dismiss: Callback<u32>,
}

#[function_component(ToastItem)]
fn toast_item(props: &ToastItemProps) -> Html {
    let id = props.toast.id;
    let on_dismiss = props.on_dismiss.clone();

    // Auto-dismiss after 4 s — hook is at component level, not inside a loop.
    use_effect_with(id, move |_| {
        let timeout = Timeout::new(4_000, move || {
            on_dismiss.emit(id);
        });
        timeout.forget();
        || {}
    });

    let on_dismiss2 = props.on_dismiss.clone();
    let dismiss = Callback::from(move |_: MouseEvent| {
        on_dismiss2.emit(id);
    });

    let class = match props.toast.kind {
        ToastKind::Success => "toast toast-success",
        ToastKind::Error   => "toast toast-error",
        ToastKind::Info    => "toast toast-info",
    };

    html! {
        <div class={class}>
            <span>{ &props.toast.message }</span>
            <button class="toast-dismiss" onclick={dismiss}>{ "\u{2715}" }</button>
        </div>
    }
}

// ── Container ─────────────────────────────────────────────────────────────────

#[function_component(ToastContainer)]
pub fn toast_container() -> Html {
    let ctx = use_context::<ToastContext>().unwrap();

    let on_dismiss = {
        let ctx2 = ctx.clone();
        Callback::from(move |id: u32| {
            ctx2.dispatch(ToastAction::Remove(id));
        })
    };

    html! {
        <div class="toast-container">
            { for ctx.toasts.iter().map(|t| {
                html! {
                    <ToastItem
                        key={t.id}
                        toast={t.clone()}
                        on_dismiss={on_dismiss.clone()}
                    />
                }
            })}
        </div>
    }
}
