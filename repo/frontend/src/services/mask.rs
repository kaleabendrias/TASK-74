//! PII masking utilities — delegates to `frontend_logic::mask` so that
//! `frontend_tests` and the frontend always exercise identical logic.

pub use frontend_logic::mask::mask_phone;
pub use frontend_logic::mask::mask_email;
