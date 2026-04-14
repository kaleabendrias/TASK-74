use chrono::Utc;
use diesel::PgConnection;
use uuid::Uuid;

use crate::errors::{ApiError, FieldError};
use crate::model::{
    CreateResourceRequest, PaginatedResponse, ResourceQuery, ResourceResponse, UserRole,
};
use crate::repository::{media, resources};
use crate::service::validation;

/// Creates a new resource after validating all fields, media refs, and scheduling.
pub fn create_resource(
    conn: &mut PgConnection,
    req: &CreateResourceRequest,
    user_id: Uuid,
    master_key: &str,
    facility_id: Option<Uuid>,
) -> Result<ResourceResponse, ApiError> {
    let mut errors = vec![];

    if let Err(e) = validation::validate_title(&req.title) {
        errors.push(e);
    }
    if let Err(e) = validation::validate_tags(&req.tags) {
        errors.push(e);
    }
    if let Err(e) = validation::validate_hours(&req.hours) {
        errors.push(e);
    }
    if let Err(e) = validation::validate_pricing(&req.pricing) {
        errors.push(e);
    }
    if req.address.is_empty() {
        errors.push(FieldError {
            field: "address".into(),
            message: "Address is required".into(),
        });
    }
    if let Err(mut e) = validation::validate_lat_lng(req.latitude, req.longitude) {
        errors.append(&mut e);
    }

    if !errors.is_empty() {
        return Err(ApiError::unprocessable_fields(
            "VALIDATION_ERROR",
            "Resource validation failed",
            errors,
        ));
    }

    // Validate media refs exist
    if !req.media_refs.is_empty() {
        let existing = media::ids_exist(conn, &req.media_refs)?;
        if existing.len() != req.media_refs.len() {
            return Err(ApiError::unprocessable(
                "INVALID_MEDIA_REFS",
                "One or more media_refs reference non-existent media files",
            ));
        }
    }

    // Encrypt contact info if provided
    let contact_encrypted = req.contact_info.as_ref().map(|info| {
        crate::crypto::aes_gcm::encrypt(info.as_bytes(), master_key)
    });

    // Parse scheduled_publish_at
    let scheduled = parse_scheduled_publish(&req.scheduled_publish_at, req.tz_offset_minutes)?;

    let new = resources::NewResource {
        title: &req.title,
        category: req.category.as_deref(),
        tags: serde_json::json!(req.tags),
        hours: req.hours.clone(),
        pricing: req.pricing.clone(),
        contact_info_encrypted: contact_encrypted,
        media_refs: serde_json::json!(req.media_refs),
        address: Some(&req.address),
        latitude: req.latitude,
        longitude: req.longitude,
        state: "draft",
        scheduled_publish_at: scheduled,
        current_version: 1,
        created_by: user_id,
        facility_id,
    };

    let row = resources::insert(conn, &new)?;
    crate::service::audit::log_action(conn, user_id, "create", "resource", Some(row.id), None, None);
    Ok(row_to_response(&row))
}

/// Retrieves a single resource by its ID.
pub fn get_resource(conn: &mut PgConnection, id: Uuid) -> Result<ResourceResponse, ApiError> {
    let row = resources::find_by_id(conn, id)?;
    Ok(row_to_response(&row))
}

/// Updates a resource, validates state transitions, and creates a version snapshot before mutation.
pub fn update_resource(
    conn: &mut PgConnection,
    id: Uuid,
    req: &crate::model::UpdateResourceRequest,
    user_id: Uuid,
    user_role: UserRole,
) -> Result<ResourceResponse, ApiError> {
    let existing = resources::find_by_id(conn, id)?;
    let mut errors = vec![];

    if let Some(ref title) = req.title {
        if let Err(e) = validation::validate_title(title) {
            errors.push(e);
        }
    }
    if let Some(ref tags) = req.tags {
        if let Err(e) = validation::validate_tags(tags) {
            errors.push(e);
        }
    }
    if let Some(ref hours) = req.hours {
        if let Err(e) = validation::validate_hours(hours) {
            errors.push(e);
        }
    }
    if let Some(ref pricing) = req.pricing {
        if let Err(e) = validation::validate_pricing(pricing) {
            errors.push(e);
        }
    }
    if let Some(ref lat) = req.latitude {
        if let Some(ref lng) = req.longitude {
            if let Err(mut e) = validation::validate_lat_lng(Some(*lat), Some(*lng)) {
                errors.append(&mut e);
            }
        }
    }

    if !errors.is_empty() {
        return Err(ApiError::unprocessable_fields(
            "VALIDATION_ERROR",
            "Resource validation failed",
            errors,
        ));
    }

    // Validate state transition
    if let Some(ref new_state) = req.state {
        validate_state_transition(&existing.state, new_state, user_role)?;
    }

    // Reviewers may only change state — reject content edits
    if user_role == UserRole::Reviewer {
        let has_content_edits = req.title.is_some()
            || req.category.is_some()
            || req.tags.is_some()
            || req.hours.is_some()
            || req.pricing.is_some()
            || req.address.is_some()
            || req.latitude.is_some()
            || req.longitude.is_some()
            || req.media_refs.is_some()
            || req.scheduled_publish_at.is_some();
        if has_content_edits {
            return Err(ApiError::forbidden(
                "Reviewers may only change the resource state. Content edits require Publisher or Administrator role."
            ));
        }
    }

    // Validate media refs
    if let Some(ref refs) = req.media_refs {
        if !refs.is_empty() {
            let existing_ids = media::ids_exist(conn, refs)?;
            if existing_ids.len() != refs.len() {
                return Err(ApiError::unprocessable(
                    "INVALID_MEDIA_REFS",
                    "One or more media_refs reference non-existent media files",
                ));
            }
        }
    }

    let scheduled = match req.scheduled_publish_at {
        Some(ref s) => Some(Some(parse_scheduled_publish(&Some(s.clone()), None)?.unwrap())),
        None => None,
    };

    // Create version snapshot before mutation
    let snapshot = serde_json::json!({
        "title": existing.title,
        "category": existing.category,
        "tags": existing.tags,
        "hours": existing.hours,
        "pricing": existing.pricing,
        "media_refs": existing.media_refs,
        "address": existing.address,
        "latitude": existing.latitude,
        "longitude": existing.longitude,
        "state": existing.state,
        "scheduled_publish_at": existing.scheduled_publish_at,
    });

    resources::insert_version(
        conn,
        &resources::NewResourceVersion {
            resource_id: id,
            version_number: existing.current_version,
            snapshot,
            changed_by: user_id,
        },
    )?;

    let changeset = resources::ResourceUpdate {
        title: req.title.clone(),
        category: req.category.as_ref().map(|c| Some(c.clone())),
        tags: req.tags.as_ref().map(|t| serde_json::json!(t)),
        hours: req.hours.clone(),
        pricing: req.pricing.clone(),
        media_refs: req.media_refs.as_ref().map(|r| serde_json::json!(r)),
        address: req.address.as_ref().map(|a| Some(a.clone())),
        latitude: req.latitude.map(Some),
        longitude: req.longitude.map(Some),
        state: req.state.clone(),
        scheduled_publish_at: scheduled,
        current_version: Some(existing.current_version + 1),
        facility_id: None,
        updated_at: Some(Utc::now()),
    };

    let updated = resources::update(conn, id, &changeset)?;
    crate::service::audit::log_action(conn, user_id, "update", "resource", Some(id),
        Some(serde_json::json!({"version": existing.current_version + 1})), None);
    Ok(row_to_response(&updated))
}

/// Lists resources with pagination, filtering, sorting, and optional facility scoping.
pub fn list_resources(
    conn: &mut PgConnection,
    query: &ResourceQuery,
    scope_facility: Option<Uuid>,
) -> Result<PaginatedResponse<ResourceResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let sort_desc = query
        .sort_order
        .as_deref()
        .map(|s| s == "desc")
        .unwrap_or(true);

    let filter = resources::ResourceFilter {
        state: query.state.clone(),
        category: query.category.clone(),
        tag: query.tag.clone(),
        created_by: None,
        facility_id: scope_facility,
        search: query.search.clone(),
        sort_by: query.sort_by.clone().unwrap_or_else(|| "created_at".to_string()),
        sort_desc,
        offset,
        limit: per_page,
    };

    let (rows, total) = resources::list_filtered(conn, &filter)?;

    Ok(PaginatedResponse {
        data: rows.iter().map(row_to_response).collect(),
        page,
        per_page,
        total,
    })
}

/// Lists the version history for a resource.
pub fn list_versions(
    conn: &mut PgConnection,
    resource_id: Uuid,
) -> Result<Vec<resources::ResourceVersionRow>, ApiError> {
    // Verify the resource exists
    resources::find_by_id(conn, resource_id)?;
    Ok(resources::list_versions(conn, resource_id)?)
}

pub fn validate_state_transition(
    current: &str,
    new: &str,
    role: UserRole,
) -> Result<(), ApiError> {
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

// All datetime inputs are treated as UTC. The frontend is responsible
// for converting local time to UTC before submission. The scheduled
// publisher evaluates against UTC time.
fn parse_scheduled_publish(
    input: &Option<String>,
    tz_offset_minutes: Option<i32>,
) -> Result<Option<chrono::DateTime<Utc>>, ApiError> {
    match input {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            let ndt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M"))
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M"))
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%m/%d/%Y %I:%M %p"))
                .map_err(|_| {
                    ApiError::unprocessable(
                        "INVALID_DATETIME",
                        "scheduled_publish_at must be a valid datetime (YYYY-MM-DDTHH:MM:SS, MM/DD/YYYY h:mm AM/PM, etc.)",
                    )
                })?;
            let utc_dt = if let Some(offset_min) = tz_offset_minutes {
                let offset = chrono::FixedOffset::east_opt(offset_min * 60)
                    .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap());
                ndt.and_local_timezone(offset)
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|| ndt.and_utc())
            } else {
                ndt.and_utc()
            };
            Ok(Some(utc_dt))
        }
    }
}

fn row_to_response(row: &resources::ResourceRow) -> ResourceResponse {
    ResourceResponse {
        id: row.id,
        title: row.title.clone(),
        category: row.category.clone(),
        tags: row.tags.clone(),
        hours: row.hours.clone(),
        pricing: row.pricing.clone(),
        media_refs: row.media_refs.clone(),
        address: row.address.clone(),
        latitude: row.latitude,
        longitude: row.longitude,
        state: row.state.clone(),
        scheduled_publish_at: row.scheduled_publish_at,
        current_version: row.current_version,
        created_by: row.created_by,
        facility_id: row.facility_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
