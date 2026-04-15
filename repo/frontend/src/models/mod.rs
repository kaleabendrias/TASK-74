//! Re-exports all shared data-transfer objects from `frontend_logic::models`.
//! Keeping the models in the shared crate ensures `frontend_tests` can import
//! and test the same type definitions that the frontend uses.

pub use frontend_logic::models::*;
