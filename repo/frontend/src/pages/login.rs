//! Login page with centered card, inline field validation, optional TOTP field on MFA challenge,
//! CSRF token stored in context memory.

use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::context::{AuthAction, AuthContext, ToastAction, ToastContext};
use crate::models::{LoginRequest, ToastKind};
use crate::router::Route;
use crate::services::api;
use frontend_logic::validation::validate_login;

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let toasts = use_context::<ToastContext>().unwrap();
    let nav = use_navigator().unwrap();

    let username = use_state(String::new);
    let password = use_state(String::new);
    let totp_code = use_state(String::new);
    let show_mfa = use_state(|| false);
    let error = use_state(|| Option::<String>::None);
    let field_errors = use_state(|| std::collections::HashMap::<String, String>::new());
    let loading = use_state(|| false);

    // Redirect if already logged in
    {
        let auth = auth.clone();
        let nav = nav.clone();
        use_effect_with(auth.user.clone(), move |user| {
            if user.is_some() {
                nav.push(&Route::Dashboard);
            }
            || {}
        });
    }

    let validate = {
        let username = username.clone();
        let password = password.clone();
        let field_errors = field_errors.clone();
        move || -> bool {
            let errs: std::collections::HashMap<String, String> =
                validate_login(&*username, &*password)
                    .into_iter()
                    .map(|(f, m)| (f.to_string(), m.to_string()))
                    .collect();
            field_errors.set(errs.clone());
            errs.is_empty()
        }
    };

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let totp_code = totp_code.clone();
        let show_mfa = show_mfa.clone();
        let error = error.clone();
        let loading = loading.clone();
        let auth = auth.clone();
        let toasts = toasts.clone();
        let nav = nav.clone();
        let field_errors = field_errors.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username = username.clone();
            let password = password.clone();
            let totp_code = totp_code.clone();
            let show_mfa = show_mfa.clone();
            let error = error.clone();
            let loading = loading.clone();
            let auth = auth.clone();
            let toasts = toasts.clone();
            let nav = nav.clone();
            let field_errors = field_errors.clone();

            // Client-side validation — delegates to frontend_logic::validation::validate_login
            let errs: std::collections::HashMap<String, String> =
                validate_login(&*username, &*password)
                    .into_iter()
                    .map(|(f, m)| (f.to_string(), m.to_string()))
                    .collect();
            field_errors.set(errs.clone());
            if !errs.is_empty() { return; }

            loading.set(true);
            error.set(None);

            spawn_local(async move {
                let req = LoginRequest {
                    username: (*username).clone(),
                    password: (*password).clone(),
                    totp_code: if *show_mfa && !totp_code.is_empty() {
                        Some((*totp_code).clone())
                    } else {
                        None
                    },
                };

                match api::login(&req).await {
                    Ok(resp) => {
                        // Check if MFA challenge
                        if resp.mfa_required == Some(true) {
                            show_mfa.set(true);
                            error.set(Some("Please enter your TOTP code".into()));
                            loading.set(false);
                            return;
                        }
                        // Fetch user profile
                        api::set_csrf_token(Some(resp.csrf_token.clone()));
                        match api::me().await {
                            Ok(user) => {
                                auth.dispatch(AuthAction::SetAuth {
                                    user,
                                    csrf_token: resp.csrf_token,
                                });
                                toasts.dispatch(ToastAction::Add(ToastKind::Success, "Welcome!".into()));
                                nav.push(&Route::Dashboard);
                            }
                            Err(e) => {
                                error.set(Some(format!("Failed to load profile: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
        })
    };

    let on_input = |setter: UseStateHandle<String>| {
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            setter.set(input.value());
        })
    };

    let fe = &*field_errors;

    html! {
        <div class="login-page">
            <div class="login-card">
                <h1>{ "Tourism Portal" }</h1>
                <p class="subtitle">{ "Sign in to your account" }</p>

                <form onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="login-username">{ "Username" }</label>
                        <input
                            id="login-username"
                            type="text"
                            class={if fe.contains_key("username") { "error" } else { "" }}
                            oninput={on_input(username.clone())}
                            value={(*username).clone()}
                            placeholder="Enter username"
                            autocomplete="username"
                        />
                        { if let Some(e) = fe.get("username") {
                            html! { <div class="field-error" id="login-username-error">{ e }</div> }
                        } else { html!{} }}
                    </div>

                    <div class="form-group">
                        <label for="login-password">{ "Password" }</label>
                        <input
                            id="login-password"
                            type="password"
                            class={if fe.contains_key("password") { "error" } else { "" }}
                            oninput={on_input(password.clone())}
                            value={(*password).clone()}
                            placeholder="Enter password"
                            autocomplete="current-password"
                        />
                        { if let Some(e) = fe.get("password") {
                            html! { <div class="field-error" id="login-password-error">{ e }</div> }
                        } else { html!{} }}
                    </div>

                    { if *show_mfa {
                        html! {
                            <div class="form-group">
                                <label for="login-totp">{ "TOTP Code" }</label>
                                <input
                                    id="login-totp"
                                    type="text"
                                    oninput={on_input(totp_code.clone())}
                                    value={(*totp_code).clone()}
                                    placeholder="6-digit code"
                                    maxlength="6"
                                    autocomplete="one-time-code"
                                />
                            </div>
                        }
                    } else { html!{} }}

                    { if let Some(ref e) = *error {
                        html! { <div class="error-banner" id="login-error">{ e }</div> }
                    } else { html!{} }}

                    <button
                        id="login-submit"
                        type="submit"
                        class="btn btn-primary btn-block"
                        disabled={*loading}
                    >
                        { if *loading { "Signing in..." } else { "Sign In" } }
                    </button>
                </form>
            </div>
        </div>
    }
}
