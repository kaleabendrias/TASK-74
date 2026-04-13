//! Application routes using yew_router. Defines all 16 navigable paths.

use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum Route {
    #[at("/")]
    Login,
    #[at("/dashboard")]
    Dashboard,
    #[at("/resources")]
    ResourceList,
    #[at("/resources/new")]
    ResourceNew,
    #[at("/resources/:id")]
    ResourceDetail { id: String },
    #[at("/resources/:id/history")]
    ResourceHistory { id: String },
    #[at("/lodgings")]
    LodgingList,
    #[at("/lodgings/new")]
    LodgingNew,
    #[at("/lodgings/:id")]
    LodgingDetail { id: String },
    #[at("/inventory")]
    Inventory,
    #[at("/inventory/transactions")]
    InventoryTransactions,
    #[at("/import-export")]
    ImportExport,
    #[at("/configuration")]
    Configuration,
    #[at("/forbidden")]
    Forbidden,
    #[not_found]
    #[at("/404")]
    NotFound,
}
