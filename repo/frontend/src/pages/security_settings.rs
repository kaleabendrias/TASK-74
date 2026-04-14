//! Security Settings page — MFA lifecycle management.
//!
//! Lets any authenticated user enrol in TOTP MFA (setup → verify → enable)
//! or disable it (requires a current TOTP code to confirm identity).

use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::context::{AuthContext, ToastAction, ToastContext};
use crate::models::{MfaSetupResponse, ToastKind};
use crate::services::api;

#[function_component(SecuritySettingsPage)]
pub fn security_settings_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let toasts = use_context::<ToastContext>().unwrap();

    // Pull the user's current MFA status from the auth context.
    let mfa_enabled = auth.user.as_ref().map(|u| u.mfa_enabled).unwrap_or(false);

    // ── Enrol state ──
    let setup_data = use_state(|| Option::<MfaSetupResponse>::None);
    let enrol_code = use_state(String::new);
    let enrol_error = use_state(|| Option::<String>::None);
    let enrol_loading = use_state(|| false);

    // ── Disable state ──
    let disable_code = use_state(String::new);
    let disable_error = use_state(|| Option::<String>::None);
    let disable_loading = use_state(|| false);

    // Fetch the setup secret when the user clicks "Begin Setup".
    let on_begin_setup = {
        let setup_data = setup_data.clone();
        let enrol_error = enrol_error.clone();
        let enrol_loading = enrol_loading.clone();
        Callback::from(move |_: MouseEvent| {
            let setup_data = setup_data.clone();
            let enrol_error = enrol_error.clone();
            let enrol_loading = enrol_loading.clone();
            enrol_loading.set(true);
            spawn_local(async move {
                match api::mfa_setup().await {
                    Ok(resp) => {
                        enrol_error.set(None);
                        setup_data.set(Some(resp));
                    }
                    Err(e) => enrol_error.set(Some(e)),
                }
                enrol_loading.set(false);
            });
        })
    };

    // Confirm the TOTP code to activate MFA.
    let on_confirm_enrol = {
        let setup_data = setup_data.clone();
        let enrol_code = enrol_code.clone();
        let enrol_error = enrol_error.clone();
        let enrol_loading = enrol_loading.clone();
        let toasts = toasts.clone();
        let auth = auth.clone();
        Callback::from(move |_: MouseEvent| {
            let secret = setup_data
                .as_ref()
                .map(|d| d.secret_base64.clone())
                .unwrap_or_default();
            let code = (*enrol_code).clone();
            let enrol_error = enrol_error.clone();
            let enrol_loading = enrol_loading.clone();
            let setup_data = setup_data.clone();
            let toasts = toasts.clone();
            let auth = auth.clone();
            enrol_loading.set(true);
            spawn_local(async move {
                match api::mfa_confirm(&secret, &code).await {
                    Ok(_) => {
                        enrol_error.set(None);
                        setup_data.set(None);
                        toasts.dispatch(ToastAction::Add(
                            ToastKind::Success,
                            "MFA enabled successfully. You will need your authenticator app at next login.".into(),
                        ));
                        // Refresh the user profile so the sidebar reflects the new MFA status.
                        if let Ok(profile) = api::me().await {
                            auth.dispatch(crate::context::AuthAction::SetUser(profile));
                        }
                    }
                    Err(e) => enrol_error.set(Some(e)),
                }
                enrol_loading.set(false);
            });
        })
    };

    // Disable MFA after verifying identity with the current TOTP code.
    let on_disable = {
        let disable_code = disable_code.clone();
        let disable_error = disable_error.clone();
        let disable_loading = disable_loading.clone();
        let toasts = toasts.clone();
        let auth = auth.clone();
        Callback::from(move |_: MouseEvent| {
            let code = (*disable_code).clone();
            let disable_error = disable_error.clone();
            let disable_loading = disable_loading.clone();
            let disable_code = disable_code.clone();
            let toasts = toasts.clone();
            let auth = auth.clone();
            disable_loading.set(true);
            spawn_local(async move {
                match api::mfa_disable(&code).await {
                    Ok(_) => {
                        disable_error.set(None);
                        disable_code.set(String::new());
                        toasts.dispatch(ToastAction::Add(
                            ToastKind::Info,
                            "MFA has been disabled.".into(),
                        ));
                        if let Ok(profile) = api::me().await {
                            auth.dispatch(crate::context::AuthAction::SetUser(profile));
                        }
                    }
                    Err(e) => disable_error.set(Some(e)),
                }
                disable_loading.set(false);
            });
        })
    };

    html! {
        <>
        <div class="page-header">
            <h1>{ "Security Settings" }</h1>
        </div>

        <div class="card">
            <div class="card-header">
                <h2>{ "Multi-Factor Authentication (TOTP)" }</h2>
            </div>

            <div class="mb-4">
                { if mfa_enabled {
                    html! { <span class="badge badge-published">{ "MFA Enabled" }</span> }
                } else {
                    html! { <span class="badge badge-draft">{ "MFA Disabled" }</span> }
                }}
            </div>

            { if !mfa_enabled {
                html! {
                    <div>
                        <p class="text-secondary text-sm mb-4">
                            { "Protect your account with a time-based one-time password (TOTP) app such as Google Authenticator or Authy." }
                        </p>

                        { if setup_data.is_none() {
                            html! {
                                <button id="btn-begin-mfa-setup" class="btn btn-primary"
                                    disabled={*enrol_loading}
                                    onclick={on_begin_setup}>
                                    { if *enrol_loading { "Loading…" } else { "Begin Setup" } }
                                </button>
                            }
                        } else {
                            let sd = setup_data.as_ref().unwrap();
                            html! {
                                <div>
                                    <p class="mb-2">{ "Enter this secret into your authenticator app:" }</p>
                                    <pre id="mfa-secret" class="text-sm mb-4"
                                        style="background:var(--color-bg);padding:8px;border-radius:4px;user-select:all;">
                                        { &sd.secret_base64 }
                                    </pre>
                                    <p class="text-secondary text-sm mb-4">
                                        { format!("Issuer: {}  |  {} digits  |  {}s period",
                                            sd.issuer, sd.digits, sd.period) }
                                    </p>
                                    <div class="form-group" style="max-width:200px;">
                                        <label for="mfa-confirm-code">{ "Verification Code" }</label>
                                        <input id="mfa-confirm-code" type="text"
                                            inputmode="numeric"
                                            maxlength="6"
                                            placeholder="6-digit code"
                                            value={(*enrol_code).clone()}
                                            oninput={{
                                                let enrol_code = enrol_code.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    enrol_code.set(input.value());
                                                })
                                            }} />
                                    </div>
                                    { if let Some(ref e) = *enrol_error {
                                        html! { <div class="field-error mb-2">{ e }</div> }
                                    } else { html!{} }}
                                    <button id="btn-confirm-mfa" class="btn btn-success"
                                        disabled={*enrol_loading}
                                        onclick={on_confirm_enrol}>
                                        { if *enrol_loading { "Verifying…" } else { "Activate MFA" } }
                                    </button>
                                </div>
                            }
                        }}

                        { if setup_data.is_none() {
                            if let Some(ref e) = *enrol_error {
                                html! { <div class="field-error mt-2">{ e }</div> }
                            } else { html!{} }
                        } else { html!{} }}
                    </div>
                }
            } else {
                html! {
                    <div>
                        <p class="text-secondary text-sm mb-4">
                            { "To disable MFA, enter your current authenticator code to confirm your identity." }
                        </p>
                        <div class="form-group" style="max-width:200px;">
                            <label for="mfa-disable-code">{ "Current TOTP Code" }</label>
                            <input id="mfa-disable-code" type="text"
                                inputmode="numeric"
                                maxlength="6"
                                placeholder="6-digit code"
                                value={(*disable_code).clone()}
                                oninput={{
                                    let disable_code = disable_code.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        disable_code.set(input.value());
                                    })
                                }} />
                        </div>
                        { if let Some(ref e) = *disable_error {
                            html! { <div class="field-error mb-2">{ e }</div> }
                        } else { html!{} }}
                        <button id="btn-disable-mfa" class="btn btn-danger"
                            disabled={*disable_loading}
                            onclick={on_disable}>
                            { if *disable_loading { "Disabling…" } else { "Disable MFA" } }
                        </button>
                    </div>
                }
            }}
        </div>
        </>
    }
}
