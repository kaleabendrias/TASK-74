//! Client-side field validators.
//!
//! These functions are called by the Yew page components before submitting API
//! requests.  Extracting them here ensures both the frontend and `frontend_tests`
//! exercise identical logic.

/// Login form: username must be non-empty; password must be ≥ 4 characters.
/// Returns a list of `(field_name, error_message)` pairs.
pub fn validate_login(username: &str, password: &str) -> Vec<(&'static str, &'static str)> {
    let mut errors = vec![];
    if username.is_empty() {
        errors.push(("username", "Username is required"));
    }
    if password.is_empty() {
        errors.push(("password", "Password is required"));
    } else if password.len() < 4 {
        errors.push(("password", "Password must be at least 4 characters"));
    }
    errors
}

/// Deposit cap rule: proposed_deposit ≤ 1.5 × monthly_rent.
/// Returns `true` when the deposit is within the allowed cap.
pub fn validate_deposit_cap(proposed_deposit: f64, monthly_rent: f64) -> bool {
    proposed_deposit <= monthly_rent * 1.5
}

/// Period nights constraint: min and max must be in [7, 365] and min ≤ max.
/// Returns a list of human-readable error strings.
pub fn validate_period_nights(min_nights: i32, max_nights: i32) -> Vec<&'static str> {
    let mut errors = vec![];
    if min_nights < 7   { errors.push("min_nights must be at least 7"); }
    if min_nights > 365 { errors.push("min_nights must not exceed 365"); }
    if max_nights < 7   { errors.push("max_nights must be at least 7"); }
    if max_nights > 365 { errors.push("max_nights must not exceed 365"); }
    if min_nights > max_nights { errors.push("min_nights must not exceed max_nights"); }
    errors
}

/// Lot quantity must be positive (> 0).
pub fn validate_lot_quantity(qty: i32) -> bool {
    qty > 0
}

/// Rent-change values must both be positive.
pub fn validate_rent_change(proposed_rent: f64, proposed_deposit: f64) -> bool {
    proposed_rent > 0.0 && proposed_deposit > 0.0
}

/// Resource title must be non-empty and at most 200 characters.
pub fn validate_resource_title(title: &str) -> bool {
    !title.is_empty() && title.len() <= 200
}
