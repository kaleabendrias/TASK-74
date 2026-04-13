//! Globally mounted toast notification container.
//! Renders success/error/info toasts with 4-second auto-dismiss.

use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::context::{ToastAction, ToastContext};
use crate::models::ToastKind;

#[function_component(ToastContainer)]
pub fn toast_container() -> Html {
    let ctx = use_context::<ToastContext>().unwrap();

    html! {
        <div class="toast-container">
            { for ctx.toasts.iter().map(|t| {
                let id = t.id;
                let ctx2 = ctx.clone();
                let class = match t.kind {
                    ToastKind::Success => "toast toast-success",
                    ToastKind::Error => "toast toast-error",
                    ToastKind::Info => "toast toast-info",
                };
                // Auto-dismiss
                let ctx3 = ctx.clone();
                use_effect_with(id, move |_| {
                    let timeout = Timeout::new(4_000, move || {
                        ctx3.dispatch(ToastAction::Remove(id));
                    });
                    timeout.forget();
                    || {}
                });

                let dismiss = Callback::from(move |_: MouseEvent| {
                    ctx2.dispatch(ToastAction::Remove(id));
                });

                html! {
                    <div key={id} class={class}>
                        <span>{ &t.message }</span>
                        <button class="toast-dismiss" onclick={dismiss}>{ "\u{2715}" }</button>
                    </div>
                }
            })}
        </div>
    }
}
