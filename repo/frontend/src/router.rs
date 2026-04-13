use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Login,
    #[at("/dashboard")]
    Dashboard,
    #[not_found]
    #[at("/404")]
    NotFound,
}
