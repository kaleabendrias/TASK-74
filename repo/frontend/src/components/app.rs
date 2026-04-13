use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{dashboard::DashboardPage, login::LoginPage};
use crate::router::Route;

fn switch(route: Route) -> Html {
    match route {
        Route::Login => html! { <LoginPage /> },
        Route::Dashboard => html! { <DashboardPage /> },
        Route::NotFound => html! { <h1>{ "404 — Not Found" }</h1> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
