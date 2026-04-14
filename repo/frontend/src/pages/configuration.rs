//! Administrator-only Configuration Center: feature toggles with confirmation modals,
//! editable config parameters, maintenance window.

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::components::route_guard::RouteGuard;
use crate::context::{ToastAction, ToastContext};
use crate::models::{ToastKind, UserRole};
use crate::services::api;

#[function_component(ConfigurationPage)]
pub fn configuration_page() -> Html {
    let toasts = use_context::<ToastContext>().unwrap();
    let config_rows = use_state(|| Vec::<serde_json::Value>::new());
    let loading = use_state(|| true);
    let confirm_modal = use_state(|| Option::<(String, String, bool)>::None); // (key, value, feature_switch)

    // Fetch config on mount
    {
        let config_rows = config_rows.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(rows) = api::list_config().await {
                    config_rows.set(rows);
                }
                loading.set(false);
            });
            || {}
        });
    }

    let save_param = {
        let config_rows = config_rows.clone();
        let toasts = toasts.clone();
        Callback::from(move |(key, value, fs): (String, String, bool)| {
            let config_rows = config_rows.clone();
            let toasts = toasts.clone();
            spawn_local(async move {
                match api::upsert_config(&key, &value, fs).await {
                    Ok(updated) => {
                        let mut rows = (*config_rows).clone();
                        if let Some(pos) = rows.iter().position(|r| r["key"].as_str() == Some(&key)) {
                            rows[pos] = updated;
                        } else {
                            rows.push(updated);
                        }
                        config_rows.set(rows);
                        toasts.dispatch(ToastAction::Add(ToastKind::Success, format!("Saved '{}'", key)));
                    }
                    Err(e) => {
                        toasts.dispatch(ToastAction::Add(ToastKind::Error, e));
                    }
                }
            });
        })
    };

    let toggle_feature = {
        let confirm_modal = confirm_modal.clone();
        let config_rows = config_rows.clone();
        Callback::from(move |key: String| {
            let current = (*config_rows).iter()
                .find(|r| r["key"].as_str() == Some(&key))
                .and_then(|r| r["feature_switch"].as_bool())
                .unwrap_or(false);
            confirm_modal.set(Some((key, (!current).to_string(), !current)));
        })
    };

    let confirm_toggle = {
        let confirm_modal = confirm_modal.clone();
        let save_param = save_param.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some((ref key, ref val, fs)) = *confirm_modal {
                save_param.emit((key.clone(), val.clone(), fs));
            }
            confirm_modal.set(None);
        })
    };

    let cancel_toggle = {
        let confirm_modal = confirm_modal.clone();
        Callback::from(move |_: MouseEvent| confirm_modal.set(None))
    };

    // Separate features (feature_switch=true) from regular params
    let features: Vec<_> = config_rows.iter()
        .filter(|r| r["feature_switch"].as_bool() == Some(true))
        .cloned().collect();
    let params: Vec<_> = config_rows.iter()
        .filter(|r| r["feature_switch"].as_bool() != Some(true))
        .cloned().collect();

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator]}>
        <>
        <div class="page-header">
            <h1>{ "Configuration Center" }</h1>
        </div>

        { if *loading {
            html! { <p class="text-secondary">{ "Loading configuration..." }</p> }
        } else {
            html! {
                <>
                <div class="card">
                    <div class="card-header"><h2>{ "Feature Switches" }</h2></div>
                    { if features.is_empty() {
                        html! { <p class="text-secondary text-sm">{ "No feature switches configured. Add one below." }</p> }
                    } else {
                        html! { for features.iter().map(|row| {
                            let key = row["key"].as_str().unwrap_or("").to_string();
                            let enabled = row["value"].as_str() == Some("true");
                            let k = key.clone();
                            let toggle = toggle_feature.clone();
                            let cls = if enabled { "toggle-switch on" } else { "toggle-switch" };
                            html! {
                                <div class="kv-row" key={key.clone()}>
                                    <span>{ &key }</span>
                                    <span class={if enabled { "badge badge-published" } else { "badge badge-offline" }}>
                                        { if enabled { "Enabled" } else { "Disabled" } }
                                    </span>
                                    <div id={format!("toggle-{}", key)} class={cls}
                                        onclick={Callback::from(move |_: MouseEvent| toggle.emit(k.clone()))} />
                                </div>
                            }
                        })}
                    }}
                </div>

                <div class="card">
                    <div class="card-header"><h2>{ "Configuration Parameters" }</h2></div>
                    { for params.iter().map(|row| {
                        let key = row["key"].as_str().unwrap_or("").to_string();
                        let value = row["value"].as_str().unwrap_or("").to_string();
                        let k2 = key.clone();
                        let save = save_param.clone();
                        html! {
                            <div class="kv-row" key={key.clone()}>
                                <span class="text-secondary text-sm">{ &key }</span>
                                <input id={format!("config-{}", key)} type="text" value={value.clone()}
                                    style="padding:6px 8px;border:1px solid var(--color-border);border-radius:var(--radius-md);font-size:0.85rem;" />
                                <button class="btn btn-sm btn-secondary"
                                    onclick={Callback::from(move |_: MouseEvent| {
                                        let doc = web_sys::window().unwrap().document().unwrap();
                                        if let Some(el) = doc.get_element_by_id(&format!("config-{}", k2)) {
                                            let input: HtmlInputElement = el.unchecked_into();
                                            save.emit((k2.clone(), input.value(), false));
                                        }
                                    })}>{ "Save" }</button>
                            </div>
                        }
                    })}
                </div>
                </>
            }
        }}

        // Confirmation modal
        { if confirm_modal.is_some() {
            html! {
                <div class="modal-overlay">
                    <div class="modal">
                        <div class="modal-header">
                            <h2>{ "Confirm Change" }</h2>
                        </div>
                        <p>{ "Are you sure you want to toggle this feature switch?" }</p>
                        <div class="modal-footer">
                            <button id="toggle-cancel" class="btn btn-secondary" onclick={cancel_toggle}>{ "Cancel" }</button>
                            <button id="toggle-confirm" class="btn btn-primary" onclick={confirm_toggle}>{ "Confirm" }</button>
                        </div>
                    </div>
                </div>
            }
        } else { html!{} }}
        </>
        </RouteGuard>
    }
}
