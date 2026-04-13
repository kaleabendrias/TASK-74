use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::auth;
use crate::models::LoginResponse;
use crate::router::Route;
use crate::services::api;

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let username = use_state(String::new);
    let password = use_state(String::new);
    let error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();

    let on_username = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_password = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let error = error.clone();
        let navigator = navigator.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let u = (*username).clone();
            let p = (*password).clone();
            let error = error.clone();
            let navigator = navigator.clone();
            spawn_local(async move {
                match api::login(&u, &p).await {
                    Ok(resp) => {
                        auth::store_tokens(&resp.session_token, &resp.csrf_token);
                        navigator.push(&Route::Dashboard);
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
            });
        })
    };

    html! {
        <div class="login-container">
            <h1>{ "Tourism Portal" }</h1>
            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="username">{ "Username" }</label>
                    <input
                        id="username"
                        type="text"
                        oninput={on_username}
                        value={(*username).clone()}
                        required=true
                    />
                </div>
                <div class="form-group">
                    <label for="password">{ "Password" }</label>
                    <input
                        id="password"
                        type="password"
                        oninput={on_password}
                        value={(*password).clone()}
                        required=true
                    />
                </div>
                <button type="submit" class="btn-primary">{ "Sign In" }</button>
                if let Some(err) = &*error {
                    <p class="error-msg">{ err }</p>
                }
            </form>
        </div>
    }
}
