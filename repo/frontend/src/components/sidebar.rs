//! Persistent sidebar navigation with role-conditional menu items, user info footer,
//! and responsive hamburger toggle below 768px.

use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::context::{AuthAction, AuthContext, ToastAction, ToastContext};
use crate::models::{ToastKind, UserRole};
use crate::router::Route;
use crate::services::api;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let toasts = use_context::<ToastContext>().unwrap();
    let nav = use_navigator().unwrap();
    let open = use_state(|| false);
    let route = use_route::<Route>();

    let user = match &auth.user {
        Some(u) => u.clone(),
        None => return html! {},
    };

    let role = &user.role;

    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };

    let close = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(false))
    };

    let on_logout = {
        let auth = auth.clone();
        let nav = nav.clone();
        let toasts = toasts.clone();
        Callback::from(move |_: MouseEvent| {
            let auth = auth.clone();
            let nav = nav.clone();
            let toasts = toasts.clone();
            spawn_local(async move {
                let _ = api::logout().await;
                auth.dispatch(AuthAction::Logout);
                toasts.dispatch(ToastAction::Add(ToastKind::Info, "Signed out".into()));
                nav.push(&Route::Login);
            });
        })
    };

    let is_active = |target: &Route| -> &'static str {
        if route.as_ref() == Some(target) { "sidebar-link active" } else { "sidebar-link" }
    };

    let initial = user.username.chars().next().unwrap_or('?').to_uppercase().to_string();

    let sidebar_class = if *open { "sidebar open" } else { "sidebar" };
    let overlay_class = if *open { "sidebar-overlay open" } else { "sidebar-overlay" };

    html! {
        <>
        <button id="hamburger-toggle" class="hamburger" onclick={toggle.clone()}>
            { "\u{2630}" }
        </button>
        <div class={overlay_class} onclick={close} />
        <aside id="sidebar" class={sidebar_class}>
            <div class="sidebar-brand">{ "Tourism Portal" }</div>
            <nav class="sidebar-nav">
                <div class="sidebar-section">{ "Main" }</div>
                <Link<Route> to={Route::Dashboard} classes={is_active(&Route::Dashboard)}>
                    { "Dashboard" }
                </Link<Route>>

                // Resources — Administrator, Publisher, Reviewer
                { if matches!(role, UserRole::Administrator | UserRole::Publisher | UserRole::Reviewer) {
                    html! {
                        <>
                        <div class="sidebar-section">{ "Content" }</div>
                        <Link<Route> to={Route::ResourceList} classes={is_active(&Route::ResourceList)}>
                            { "Resources" }
                        </Link<Route>>
                        <Link<Route> to={Route::LodgingList} classes={is_active(&Route::LodgingList)}>
                            { "Lodgings" }
                        </Link<Route>>
                        </>
                    }
                } else { html!{} }}

                // Inventory — Administrator, Clinician, InventoryClerk
                { if matches!(role, UserRole::Administrator | UserRole::Clinician | UserRole::InventoryClerk) {
                    html! {
                        <>
                        <div class="sidebar-section">{ "Inventory" }</div>
                        <Link<Route> to={Route::Inventory} classes={is_active(&Route::Inventory)}>
                            { "Stock" }
                        </Link<Route>>
                        <Link<Route> to={Route::InventoryTransactions} classes={is_active(&Route::InventoryTransactions)}>
                            { "Transactions" }
                        </Link<Route>>
                        </>
                    }
                } else { html!{} }}

                // Import/Export — Administrator, InventoryClerk
                { if matches!(role, UserRole::Administrator | UserRole::InventoryClerk) {
                    html! {
                        <>
                        <div class="sidebar-section">{ "Data" }</div>
                        <Link<Route> to={Route::ImportExport} classes={is_active(&Route::ImportExport)}>
                            { "Import / Export" }
                        </Link<Route>>
                        </>
                    }
                } else { html!{} }}

                // Configuration — Administrator only
                { if matches!(role, UserRole::Administrator) {
                    html! {
                        <>
                        <div class="sidebar-section">{ "System" }</div>
                        <Link<Route> to={Route::Configuration} classes={is_active(&Route::Configuration)}>
                            { "Configuration" }
                        </Link<Route>>
                        </>
                    }
                } else { html!{} }}
            </nav>

            <div class="sidebar-footer">
                <div class="sidebar-user">
                    <div class="sidebar-avatar">{ &initial }</div>
                    <div class="sidebar-user-info">
                        <div class="sidebar-user-name">{ &user.username }</div>
                        <div class="sidebar-user-role">{ user.role.to_string() }</div>
                    </div>
                </div>
                <button id="logout-btn" class="btn btn-secondary btn-sm btn-block mt-4"
                    onclick={on_logout}>
                    { "Sign Out" }
                </button>
            </div>
        </aside>
        </>
    }
}
