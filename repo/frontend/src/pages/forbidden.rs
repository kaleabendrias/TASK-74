//! 403 Forbidden and 404 Not Found pages with navigation back to dashboard.

use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

#[function_component(ForbiddenPage)]
pub fn forbidden_page() -> Html {
    html! {
        <div class="forbidden-page">
            <h1>{ "403" }</h1>
            <p>{ "You do not have permission to access this page." }</p>
            <Link<Route> to={Route::Dashboard} classes="btn btn-primary mt-6">
                { "Back to Dashboard" }
            </Link<Route>>
        </div>
    }
}

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div class="forbidden-page">
            <h1>{ "404" }</h1>
            <p>{ "Page not found." }</p>
            <Link<Route> to={Route::Dashboard} classes="btn btn-primary mt-6">
                { "Back to Dashboard" }
            </Link<Route>>
        </div>
    }
}
