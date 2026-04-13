//! Administrator-only Configuration Center: feature toggles with confirmation modals,
//! editable config parameters, maintenance window.

use yew::prelude::*;

use crate::components::route_guard::RouteGuard;
use crate::models::UserRole;

#[function_component(ConfigurationPage)]
pub fn configuration_page() -> Html {
    let features = use_state(|| vec![
        ("mfa_enabled".to_string(), true),
        ("csv_import".to_string(), true),
        ("export_watermark".to_string(), true),
        ("lodging_deposit_cap".to_string(), true),
        ("canary_release".to_string(), false),
    ]);

    let config_params = use_state(|| vec![
        ("app.config_profile".to_string(), "development".to_string()),
        ("maintenance.window_cron".to_string(), "0 3 * * 0".to_string()),
        ("prometheus.scrape_path".to_string(), "/metrics".to_string()),
        ("uploads.max_size_bytes".to_string(), "52428800".to_string()),
        ("totp.issuer".to_string(), "TourismPortal".to_string()),
    ]);

    let confirm_modal = use_state(|| Option::<(String, bool)>::None);

    let toggle_feature = {
        let features = features.clone();
        let confirm_modal = confirm_modal.clone();
        Callback::from(move |key: String| {
            let mut f = (*features).clone();
            if let Some(entry) = f.iter_mut().find(|(k, _)| k == &key) {
                confirm_modal.set(Some((key, !entry.1)));
            }
        })
    };

    let confirm_toggle = {
        let features = features.clone();
        let confirm_modal = confirm_modal.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some((ref key, val)) = *confirm_modal {
                let mut f = (*features).clone();
                if let Some(entry) = f.iter_mut().find(|(k, _)| k == key) {
                    entry.1 = val;
                }
                features.set(f);
            }
            confirm_modal.set(None);
        })
    };

    let cancel_toggle = {
        let confirm_modal = confirm_modal.clone();
        Callback::from(move |_: MouseEvent| confirm_modal.set(None))
    };

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator]}>
        <>
        <div class="page-header">
            <h1>{ "Configuration Center" }</h1>
        </div>

        <div class="card">
            <div class="card-header"><h2>{ "Feature Switches" }</h2></div>
            { for features.iter().map(|(key, enabled)| {
                let k = key.clone();
                let toggle = toggle_feature.clone();
                let cls = if *enabled { "toggle-switch on" } else { "toggle-switch" };
                html! {
                    <div class="kv-row" key={key.clone()}>
                        <span>{ key }</span>
                        <span class={if *enabled { "badge badge-published" } else { "badge badge-offline" }}>
                            { if *enabled { "Enabled" } else { "Disabled" } }
                        </span>
                        <div id={format!("toggle-{}", key)} class={cls}
                            onclick={Callback::from(move |_: MouseEvent| toggle.emit(k.clone()))} />
                    </div>
                }
            })}
        </div>

        <div class="card">
            <div class="card-header"><h2>{ "Configuration Parameters" }</h2></div>
            { for config_params.iter().map(|(key, value)| {
                html! {
                    <div class="kv-row" key={key.clone()}>
                        <span class="text-secondary text-sm">{ key }</span>
                        <input id={format!("config-{}", key)} type="text" value={value.clone()}
                            class="" style="padding:6px 8px;border:1px solid var(--color-border);border-radius:var(--radius-md);font-size:0.85rem;" />
                        <button class="btn btn-sm btn-secondary">{ "Save" }</button>
                    </div>
                }
            })}
        </div>

        <div class="card">
            <div class="card-header"><h2>{ "Maintenance Window" }</h2></div>
            <div class="form-group">
                <label for="maint-cron">{ "Cron expression" }</label>
                <input id="maint-cron" type="text" value="0 3 * * 0" />
                <div class="text-secondary text-sm mt-2">{ "Currently: Sundays at 3:00 AM" }</div>
            </div>
        </div>

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
