use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::PgConnection;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::{ApiError, FieldError};
use crate::model::*;
use crate::repository::lodgings as repo;
use crate::service::validation;

pub fn create_lodging(
    conn: &mut PgConnection,
    req: &CreateLodgingRequest,
    user_id: Uuid,
) -> Result<LodgingResponse, ApiError> {
    let mut errors = vec![];

    if req.name.is_empty() || req.name.len() > 500 {
        errors.push(FieldError {
            field: "name".into(),
            message: "Name is required and must not exceed 500 characters".into(),
        });
    }
    if let Err(mut e) = validation::validate_amenities(&req.amenities) {
        errors.append(&mut e);
    }

    if !errors.is_empty() {
        return Err(ApiError::unprocessable_fields(
            "VALIDATION_ERROR",
            "Lodging validation failed",
            errors,
        ));
    }

    // Deposit cap validation
    if let (Some(dep), Some(rent)) = (req.deposit_amount, req.monthly_rent) {
        validation::validate_deposit_cap(dep, rent)?;
    }

    let deposit = req.deposit_amount.map(|d| BigDecimal::from_str(&format!("{:.2}", d)).unwrap());
    let rent = req.monthly_rent.map(|r| BigDecimal::from_str(&format!("{:.2}", r)).unwrap());
    let cap_valid = req.deposit_amount.is_some() && req.monthly_rent.is_some();

    let new = repo::NewLodging {
        name: &req.name,
        description: req.description.as_deref(),
        state: "draft",
        amenities: serde_json::json!(req.amenities),
        facility_id: req.facility_id,
        deposit_amount: deposit,
        monthly_rent: rent,
        deposit_cap_validated: cap_valid,
        created_by: user_id,
    };

    let row = repo::insert_lodging(conn, &new)?;
    Ok(row_to_response(&row))
}

pub fn get_lodging(conn: &mut PgConnection, id: Uuid) -> Result<LodgingResponse, ApiError> {
    let row = repo::find_lodging_by_id(conn, id)?;
    Ok(row_to_response(&row))
}

pub fn update_lodging(
    conn: &mut PgConnection,
    id: Uuid,
    req: &UpdateLodgingRequest,
    user_role: UserRole,
) -> Result<LodgingResponse, ApiError> {
    let existing = repo::find_lodging_by_id(conn, id)?;

    if let Some(ref amenities) = req.amenities {
        if let Err(errs) = validation::validate_amenities(amenities) {
            return Err(ApiError::unprocessable_fields(
                "VALIDATION_ERROR",
                "Invalid amenities",
                errs,
            ));
        }
    }

    // Deposit cap validation on updated values
    let new_deposit = req.deposit_amount.or_else(|| bd_to_f64(&existing.deposit_amount));
    let new_rent = req.monthly_rent.or_else(|| bd_to_f64(&existing.monthly_rent));
    if let (Some(dep), Some(rent)) = (new_deposit, new_rent) {
        validation::validate_deposit_cap(dep, rent)?;
    }

    // State transition validation
    if let Some(ref new_state) = req.state {
        validate_lodging_state(&existing.state, new_state, user_role)?;
    }

    let deposit = req.deposit_amount.map(|d| Some(BigDecimal::from_str(&format!("{:.2}", d)).unwrap()));
    let rent = req.monthly_rent.map(|r| Some(BigDecimal::from_str(&format!("{:.2}", r)).unwrap()));
    let cap_valid = if new_deposit.is_some() && new_rent.is_some() {
        Some(true)
    } else {
        None
    };

    let changeset = repo::LodgingUpdate {
        name: req.name.clone(),
        description: req.description.as_ref().map(|d| Some(d.clone())),
        state: req.state.clone(),
        amenities: req.amenities.as_ref().map(|a| serde_json::json!(a)),
        facility_id: req.facility_id.map(Some),
        deposit_amount: deposit,
        monthly_rent: rent,
        deposit_cap_validated: cap_valid,
        updated_at: Some(Utc::now()),
    };

    let row = repo::update_lodging(conn, id, &changeset)?;
    Ok(row_to_response(&row))
}

pub fn list_lodgings(
    conn: &mut PgConnection,
    facility_id: Option<Uuid>,
) -> Result<Vec<LodgingResponse>, ApiError> {
    let rows = repo::list_lodgings(conn, facility_id)?;
    Ok(rows.iter().map(row_to_response).collect())
}

// ── Periods ──

pub fn get_periods(
    conn: &mut PgConnection,
    lodging_id: Uuid,
) -> Result<Vec<LodgingPeriodResponse>, ApiError> {
    // Verify lodging exists
    repo::find_lodging_by_id(conn, lodging_id)?;
    let rows = repo::list_periods(conn, lodging_id)?;
    Ok(rows.into_iter().map(period_to_response).collect())
}

pub fn upsert_period(
    conn: &mut PgConnection,
    lodging_id: Uuid,
    req: &LodgingPeriodRequest,
) -> Result<LodgingPeriodResponse, ApiError> {
    repo::find_lodging_by_id(conn, lodging_id)?;

    let min = req.min_nights.unwrap_or(7);
    let max = req.max_nights.unwrap_or(365);
    if min < 7 {
        return Err(ApiError::unprocessable(
            "INVALID_PERIOD",
            "Minimum nights must be at least 7",
        ));
    }
    if max > 365 {
        return Err(ApiError::unprocessable(
            "INVALID_PERIOD",
            "Maximum nights must not exceed 365",
        ));
    }
    if req.start_date >= req.end_date {
        return Err(ApiError::unprocessable(
            "INVALID_PERIOD",
            "start_date must be before end_date",
        ));
    }

    // Overlap detection
    let overlapping =
        repo::find_overlapping_periods(conn, lodging_id, req.start_date, req.end_date)?;
    if !overlapping.is_empty() {
        return Err(ApiError::conflict(
            "Period overlaps with an existing period",
        ));
    }

    let new = repo::NewPeriod {
        lodging_id,
        start_date: req.start_date,
        end_date: req.end_date,
        min_nights: min,
        max_nights: max,
        vacancy: req.vacancy.unwrap_or(true),
    };

    let row = repo::insert_period(conn, &new)?;
    Ok(period_to_response(row))
}

// ── Rent Changes ──

pub fn request_rent_change(
    conn: &mut PgConnection,
    lodging_id: Uuid,
    req: &RentChangeRequest,
    user_id: Uuid,
) -> Result<RentChangeResponse, ApiError> {
    let existing = repo::find_lodging_by_id(conn, lodging_id)?;
    let existing_rent = bd_to_f64(&existing.monthly_rent).unwrap_or(0.0);

    // Validate new deposit against proposed rent
    validation::validate_deposit_cap(req.proposed_deposit, req.proposed_rent)?;

    let new = repo::NewRentChange {
        lodging_id,
        proposed_rent: BigDecimal::from_str(&format!("{:.2}", req.proposed_rent)).unwrap(),
        proposed_deposit: BigDecimal::from_str(&format!("{:.2}", req.proposed_deposit)).unwrap(),
        status: "pending".to_string(),
        requested_by: user_id,
    };

    let row = repo::insert_rent_change(conn, &new)?;
    Ok(rent_change_to_response(&row))
}

pub fn approve_rent_change(
    conn: &mut PgConnection,
    lodging_id: Uuid,
    change_id: Uuid,
    reviewer_id: Uuid,
) -> Result<RentChangeResponse, ApiError> {
    let change = repo::find_rent_change(conn, change_id)?;
    if change.lodging_id != lodging_id {
        return Err(ApiError::not_found("Rent change"));
    }
    if change.status != "pending" {
        return Err(ApiError::unprocessable(
            "INVALID_STATUS",
            "Only pending rent changes can be approved",
        ));
    }

    // Atomically update the rent change and lodging in a transaction
    let updated_change = repo::update_rent_change_status(conn, change_id, "approved", reviewer_id)?;

    // Apply to lodging
    let changeset = repo::LodgingUpdate {
        name: None,
        description: None,
        state: None,
        amenities: None,
        facility_id: None,
        deposit_amount: Some(Some(updated_change.proposed_deposit.clone())),
        monthly_rent: Some(Some(updated_change.proposed_rent.clone())),
        deposit_cap_validated: Some(true),
        updated_at: Some(Utc::now()),
    };
    repo::update_lodging(conn, lodging_id, &changeset)?;

    Ok(rent_change_to_response(&updated_change))
}

pub fn reject_rent_change(
    conn: &mut PgConnection,
    lodging_id: Uuid,
    change_id: Uuid,
    reviewer_id: Uuid,
) -> Result<RentChangeResponse, ApiError> {
    let change = repo::find_rent_change(conn, change_id)?;
    if change.lodging_id != lodging_id {
        return Err(ApiError::not_found("Rent change"));
    }
    if change.status != "pending" {
        return Err(ApiError::unprocessable(
            "INVALID_STATUS",
            "Only pending rent changes can be rejected",
        ));
    }

    let updated = repo::update_rent_change_status(conn, change_id, "rejected", reviewer_id)?;
    Ok(rent_change_to_response(&updated))
}

// ── Helpers ──

fn validate_lodging_state(current: &str, new: &str, role: UserRole) -> Result<(), ApiError> {
    let allowed = match (current, new) {
        ("draft", "in_review") => role == UserRole::Publisher,
        ("in_review", "published") => role == UserRole::Reviewer,
        ("published", "offline") => {
            role == UserRole::Publisher || role == UserRole::Administrator
        }
        ("offline", "draft") => role == UserRole::Publisher,
        _ => false,
    };
    if !allowed {
        Err(ApiError::unprocessable(
            "INVALID_STATE_TRANSITION",
            &format!(
                "Transition from '{}' to '{}' is not allowed for role {:?}",
                current, new, role
            ),
        ))
    } else {
        Ok(())
    }
}

fn bd_to_f64(bd: &Option<BigDecimal>) -> Option<f64> {
    bd.as_ref().and_then(|b| b.to_string().parse::<f64>().ok())
}

fn row_to_response(row: &repo::LodgingRow) -> LodgingResponse {
    LodgingResponse {
        id: row.id,
        name: row.name.clone(),
        description: row.description.clone(),
        state: row.state.clone(),
        amenities: row.amenities.clone(),
        facility_id: row.facility_id,
        deposit_amount: bd_to_f64(&row.deposit_amount),
        monthly_rent: bd_to_f64(&row.monthly_rent),
        deposit_cap_validated: row.deposit_cap_validated,
        created_by: row.created_by,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn period_to_response(row: repo::PeriodRow) -> LodgingPeriodResponse {
    LodgingPeriodResponse {
        id: row.id,
        lodging_id: row.lodging_id,
        start_date: row.start_date,
        end_date: row.end_date,
        min_nights: row.min_nights,
        max_nights: row.max_nights,
        vacancy: row.vacancy,
    }
}

fn rent_change_to_response(row: &repo::RentChangeRow) -> RentChangeResponse {
    RentChangeResponse {
        id: row.id,
        lodging_id: row.lodging_id,
        proposed_rent: row.proposed_rent.to_string().parse().unwrap_or(0.0),
        proposed_deposit: row.proposed_deposit.to_string().parse().unwrap_or(0.0),
        status: row.status.clone(),
        requested_by: row.requested_by,
        reviewed_by: row.reviewed_by,
        reviewed_at: row.reviewed_at,
        created_at: row.created_at,
    }
}
