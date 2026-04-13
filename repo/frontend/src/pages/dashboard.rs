use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::context::AuthContext;
use crate::models::HealthResponse;
use crate::services::api;

#[function_component(DashboardPage)]
pub fn dashboard_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let health = use_state(|| Option::<HealthResponse>::None);

    {
        let health = health.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(h) = api::health().await {
                    health.set(Some(h));
                }
            });
            || {}
        });
    }

    let user = match &auth.user {
        Some(u) => u,
        None => return html! { <p>{ "Loading..." }</p> },
    };

    html! {
        <>
        <div class="page-header">
            <h1>{ format!("Welcome, {}", user.username) }</h1>
        </div>

        <div class="card">
            <div class="card-header">
                <h2>{ "System Health" }</h2>
            </div>
            { if let Some(ref h) = *health {
                html! {
                    <div>
                        <div class="kv-row">
                            <span class="text-secondary">{ "Service" }</span>
                            <span>{ &h.service }</span>
                            <span />
                        </div>
                        <div class="kv-row">
                            <span class="text-secondary">{ "Version" }</span>
                            <span>{ &h.version }</span>
                            <span />
                        </div>
                        <div class="kv-row">
                            <span class="text-secondary">{ "Uptime" }</span>
                            <span>{ format!("{} seconds", h.uptime_secs) }</span>
                            <span />
                        </div>
                        <div class="kv-row">
                            <span class="text-secondary">{ "Database" }</span>
                            <span>
                                <span class={if h.database_connected { "badge badge-published" } else { "badge badge-offline" }}>
                                    { if h.database_connected { "Connected" } else { "Disconnected" } }
                                </span>
                            </span>
                            <span />
                        </div>
                        <div class="kv-row">
                            <span class="text-secondary">{ "Config Profile" }</span>
                            <span>{ &h.config_profile }</span>
                            <span />
                        </div>
                    </div>
                }
            } else {
                html! { <p class="text-secondary">{ "Loading health data..." }</p> }
            }}
        </div>
        </>
    }
}
