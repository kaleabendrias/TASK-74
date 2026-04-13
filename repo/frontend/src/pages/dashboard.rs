use yew::prelude::*;
use yew_router::prelude::*;

use crate::auth;
use crate::router::Route;

#[function_component(DashboardPage)]
pub fn dashboard_page() -> Html {
    let navigator = use_navigator().unwrap();

    if !auth::is_authenticated() {
        navigator.push(&Route::Login);
    }

    let on_logout = {
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            auth::clear_tokens();
            navigator.push(&Route::Login);
        })
    };

    html! {
        <div style="padding: 2rem;">
            <h1>{ "Dashboard" }</h1>
            <p>{ "Welcome to the Tourism Portal." }</p>
            <button onclick={on_logout} class="btn-primary" style="max-width: 200px; margin-top: 1rem;">
                { "Sign Out" }
            </button>
        </div>
    }
}
