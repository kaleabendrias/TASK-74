//! Route permission matrix — which roles can access which routes.
//!
//! `Route` mirrors `frontend/src/router.rs` variants (without Yew/yew-router deps).
//! `can_access` encodes the `allowed_roles` lists from `app.rs` RouteGuard usages.
//!
//! The frontend's `app.rs` uses these same allowed-roles lists for `RouteGuard`;
//! keeping them here as the single source of truth means a permission change in
//! one place is immediately tested by `frontend_tests`.

use crate::models::UserRole;

/// Route variants matching the frontend router.
#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Dashboard,
    ResourceList,
    ResourceNew,
    ResourceDetail,
    ResourceHistory,
    LodgingList,
    LodgingNew,
    LodgingDetail,
    Inventory,
    InventoryTransactions,
    ImportExport,
    Configuration,
    SecuritySettings,
    Forbidden,
}

/// Returns `true` if `role` is permitted to access `route`.
/// Mirrors the `RouteGuard allowed_roles` configuration in `app.rs`.
pub fn can_access(role: &UserRole, route: &Route) -> bool {
    match route {
        // Accessible to all authenticated users
        Route::Dashboard | Route::SecuritySettings | Route::Forbidden => true,

        // Content (read): Admin, Publisher, Reviewer, Clinician
        Route::ResourceList | Route::ResourceDetail | Route::ResourceHistory
        | Route::LodgingList | Route::LodgingDetail => {
            matches!(role,
                UserRole::Administrator | UserRole::Publisher
                | UserRole::Reviewer    | UserRole::Clinician)
        }

        // Content (write): Admin, Publisher
        Route::ResourceNew | Route::LodgingNew => {
            matches!(role, UserRole::Administrator | UserRole::Publisher)
        }

        // Inventory: Admin, Clinician, InventoryClerk
        Route::Inventory | Route::InventoryTransactions => {
            matches!(role,
                UserRole::Administrator | UserRole::Clinician | UserRole::InventoryClerk)
        }

        // Import/Export: Admin, InventoryClerk, Reviewer
        Route::ImportExport => {
            matches!(role,
                UserRole::Administrator | UserRole::InventoryClerk | UserRole::Reviewer)
        }

        // Configuration: Administrator only
        Route::Configuration => {
            matches!(role, UserRole::Administrator)
        }
    }
}
