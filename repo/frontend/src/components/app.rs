use yew::prelude::*;
use yew_router::prelude::*;

use crate::context::{AuthContext, AuthProvider, ToastProvider};
use crate::pages::*;
use crate::router::Route;

use super::sidebar::Sidebar;
use super::toast::ToastContainer;

fn switch_route(route: Route) -> Html {
    match route {
        Route::Login => html! { <login::LoginPage /> },
        Route::Dashboard => html! { <dashboard::DashboardPage /> },
        Route::ResourceList => html! { <resources::ResourceListPage /> },
        Route::ResourceNew => html! { <resources::ResourceFormPage /> },
        Route::ResourceDetail { id } => html! { <resources::ResourceFormPage id={Some(id)} /> },
        Route::ResourceHistory { id } => html! { <resources::ResourceHistoryPage {id} /> },
        Route::LodgingList => html! { <lodgings::LodgingListPage /> },
        Route::LodgingNew => html! { <lodgings::LodgingFormPage /> },
        Route::LodgingDetail { id } => html! { <lodgings::LodgingFormPage id={Some(id)} /> },
        Route::Inventory => html! { <inventory::InventoryPage /> },
        Route::InventoryTransactions => html! { <inventory::TransactionsPage /> },
        Route::ImportExport => html! { <import_export::ImportExportPage /> },
        Route::Configuration => html! { <configuration::ConfigurationPage /> },
        Route::Forbidden => html! { <forbidden::ForbiddenPage /> },
        Route::NotFound => html! { <forbidden::NotFoundPage /> },
    }
}

#[function_component(AppInner)]
fn app_inner() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let route = use_route::<Route>();

    let is_login = matches!(route, Some(Route::Login) | None);
    let is_authed = auth.user.is_some();

    if is_login || !is_authed {
        html! {
            <Switch<Route> render={switch_route} />
        }
    } else {
        html! {
            <div class="shell">
                <Sidebar />
                <main class="main-content">
                    <Switch<Route> render={switch_route} />
                </main>
            </div>
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <AuthProvider>
            <ToastProvider>
                <BrowserRouter>
                    <AppInner />
                    <ToastContainer />
                </BrowserRouter>
            </ToastProvider>
        </AuthProvider>
    }
}
