use yew::prelude::*;
use yew_router::prelude::*;

use crate::context::AuthContext;
use crate::models::UserRole;
use crate::router::Route;

#[derive(Properties, PartialEq)]
pub struct GuardProps {
    pub allowed_roles: Vec<UserRole>,
    pub children: Children,
}

#[function_component(RouteGuard)]
pub fn route_guard(props: &GuardProps) -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let nav = use_navigator().unwrap();

    if let Some(ref user) = auth.user {
        if props.allowed_roles.contains(&user.role) {
            return html! { { props.children.clone() } };
        }
    }

    // Redirect to forbidden
    nav.push(&Route::Forbidden);
    html! {}
}
