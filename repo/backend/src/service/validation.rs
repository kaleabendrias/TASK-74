use crate::errors::{ApiError, FieldError};

/// The canonical set of amenity identifiers accepted by the system.
pub const ALLOWED_AMENITIES: &[&str] = &[
    "wifi", "parking", "pool", "gym", "air_conditioning", "heating",
    "kitchen", "laundry", "elevator", "wheelchair_accessible",
    "pet_friendly", "balcony", "garden", "security", "cctv",
    "reception_24h", "room_service", "restaurant", "bar", "spa",
];

/// Validates that a title is non-empty and does not exceed 200 characters.
pub fn validate_title(title: &str) -> Result<(), FieldError> {
    if title.is_empty() {
        return Err(FieldError {
            field: "title".into(),
            message: "Title is required".into(),
        });
    }
    if title.len() > 200 {
        return Err(FieldError {
            field: "title".into(),
            message: "Title must not exceed 200 characters".into(),
        });
    }
    Ok(())
}

/// Validates that the tags array does not exceed 20 items.
pub fn validate_tags(tags: &[String]) -> Result<(), FieldError> {
    if tags.len() > 20 {
        return Err(FieldError {
            field: "tags".into(),
            message: "Tags array must not exceed 20 items".into(),
        });
    }
    Ok(())
}

/// Validates that all pricing values in the JSON object are non-negative.
pub fn validate_pricing(pricing: &serde_json::Value) -> Result<(), FieldError> {
    if let Some(obj) = pricing.as_object() {
        for (key, val) in obj {
            if let Some(n) = val.as_f64() {
                if n < 0.0 {
                    return Err(FieldError {
                        field: format!("pricing.{}", key),
                        message: "Pricing values must be non-negative".into(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Validates that latitude is in [-90, 90] and longitude is in [-180, 180].
pub fn validate_lat_lng(lat: Option<f64>, lng: Option<f64>) -> Result<(), Vec<FieldError>> {
    let mut errs = vec![];
    if let Some(lat) = lat {
        if !(-90.0..=90.0).contains(&lat) {
            errs.push(FieldError {
                field: "latitude".into(),
                message: "Latitude must be between -90 and 90".into(),
            });
        }
    }
    if let Some(lng) = lng {
        if !(-180.0..=180.0).contains(&lng) {
            errs.push(FieldError {
                field: "longitude".into(),
                message: "Longitude must be between -180 and 180".into(),
            });
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Validates that hours is either null or a JSON object mapping day names to open/close times.
pub fn validate_hours(hours: &serde_json::Value) -> Result<(), FieldError> {
    // Accept structured JSON: {"monday": {"open": "09:00", "close": "17:00"}, ...}
    if hours.is_null() || hours.is_object() {
        Ok(())
    } else {
        Err(FieldError {
            field: "hours".into(),
            message: "Hours must be a JSON object mapping day names to open/close times".into(),
        })
    }
}

/// Validates that all provided amenities are in the [`ALLOWED_AMENITIES`] set.
pub fn validate_amenities(amenities: &[String]) -> Result<(), Vec<FieldError>> {
    let mut errs = vec![];
    for a in amenities {
        if !ALLOWED_AMENITIES.contains(&a.as_str()) {
            errs.push(FieldError {
                field: "amenities".into(),
                message: format!("Unknown amenity '{}'. Allowed: {}", a, ALLOWED_AMENITIES.join(", ")),
            });
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Ensures the deposit does not exceed 1.5x the monthly rent.
pub fn validate_deposit_cap(deposit: f64, monthly_rent: f64) -> Result<(), ApiError> {
    let cap = monthly_rent * 1.5;
    if deposit > cap {
        Err(ApiError::unprocessable(
            "DEPOSIT_CAP_EXCEEDED",
            &format!(
                "Deposit ({:.2}) cannot exceed 1.5x monthly rent ({:.2}). Maximum allowed: {:.2}",
                deposit, monthly_rent, cap
            ),
        ))
    } else {
        Ok(())
    }
}
