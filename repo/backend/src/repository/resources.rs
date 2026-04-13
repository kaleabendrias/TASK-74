use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{resources, resource_versions};

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = resources)]
pub struct ResourceRow {
    pub id: Uuid,
    pub title: String,
    pub category: Option<String>,
    pub tags: serde_json::Value,
    pub hours: serde_json::Value,
    pub pricing: serde_json::Value,
    pub contact_info_encrypted: Option<Vec<u8>>,
    pub media_refs: serde_json::Value,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub state: String,
    pub scheduled_publish_at: Option<DateTime<Utc>>,
    pub current_version: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = resources)]
pub struct NewResource<'a> {
    pub title: &'a str,
    pub category: Option<&'a str>,
    pub tags: serde_json::Value,
    pub hours: serde_json::Value,
    pub pricing: serde_json::Value,
    pub media_refs: serde_json::Value,
    pub address: Option<&'a str>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub state: &'a str,
    pub scheduled_publish_at: Option<DateTime<Utc>>,
    pub current_version: i32,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = resources)]
pub struct ResourceUpdate {
    pub title: Option<String>,
    pub category: Option<Option<String>>,
    pub tags: Option<serde_json::Value>,
    pub hours: Option<serde_json::Value>,
    pub pricing: Option<serde_json::Value>,
    pub media_refs: Option<serde_json::Value>,
    pub address: Option<Option<String>>,
    pub latitude: Option<Option<f64>>,
    pub longitude: Option<Option<f64>>,
    pub state: Option<String>,
    pub scheduled_publish_at: Option<Option<DateTime<Utc>>>,
    pub current_version: Option<i32>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Inserts a new resource into the database.
pub fn insert(conn: &mut PgConnection, new: &NewResource) -> QueryResult<ResourceRow> {
    diesel::insert_into(resources::table)
        .values(new)
        .returning(ResourceRow::as_returning())
        .get_result(conn)
}

/// Finds a resource by its unique ID.
pub fn find_by_id(conn: &mut PgConnection, rid: Uuid) -> QueryResult<ResourceRow> {
    resources::table
        .find(rid)
        .select(ResourceRow::as_select())
        .first(conn)
}

/// Applies a partial update to a resource and returns the updated row.
pub fn update(
    conn: &mut PgConnection,
    rid: Uuid,
    changeset: &ResourceUpdate,
) -> QueryResult<ResourceRow> {
    diesel::update(resources::table.find(rid))
        .set(changeset)
        .returning(ResourceRow::as_returning())
        .get_result(conn)
}

pub struct ResourceFilter {
    pub state: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub created_by: Option<Uuid>,
    pub search: Option<String>,
    pub sort_by: String,
    pub sort_desc: bool,
    pub offset: i64,
    pub limit: i64,
}

/// Lists resources matching the given filter criteria with pagination.
pub fn list_filtered(
    conn: &mut PgConnection,
    filter: &ResourceFilter,
) -> QueryResult<(Vec<ResourceRow>, i64)> {
    let mut query = resources::table.into_boxed();
    let mut count_query = resources::table.into_boxed();

    if let Some(ref s) = filter.state {
        query = query.filter(resources::state.eq(s));
        count_query = count_query.filter(resources::state.eq(s));
    }
    if let Some(ref c) = filter.category {
        query = query.filter(resources::category.eq(c));
        count_query = count_query.filter(resources::category.eq(c));
    }
    if let Some(ref uid) = filter.created_by {
        query = query.filter(resources::created_by.eq(uid));
        count_query = count_query.filter(resources::created_by.eq(uid));
    }
    if let Some(ref s) = filter.search {
        let pattern = format!("%{}%", s);
        query = query.filter(resources::title.ilike(pattern.clone()));
        count_query = count_query.filter(resources::title.ilike(pattern));
    }

    let total: i64 = count_query.count().get_result(conn)?;

    query = match filter.sort_by.as_str() {
        "scheduled_publish_at" => {
            if filter.sort_desc {
                query.order(resources::scheduled_publish_at.desc())
            } else {
                query.order(resources::scheduled_publish_at.asc())
            }
        }
        _ => {
            if filter.sort_desc {
                query.order(resources::created_at.desc())
            } else {
                query.order(resources::created_at.asc())
            }
        }
    };

    let rows = query
        .offset(filter.offset)
        .limit(filter.limit)
        .select(ResourceRow::as_select())
        .load(conn)?;

    // Post-filter by tag in Rust since JSONB array contains is awkward with boxed queries
    let rows = if let Some(ref tag) = filter.tag {
        rows.into_iter()
            .filter(|r| {
                r.tags
                    .as_array()
                    .map(|arr| arr.iter().any(|v| v.as_str() == Some(tag)))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        rows
    };

    Ok((rows, total))
}

// ── Versions ──

#[derive(Insertable)]
#[diesel(table_name = resource_versions)]
pub struct NewResourceVersion {
    pub resource_id: Uuid,
    pub version_number: i32,
    pub snapshot: serde_json::Value,
    pub changed_by: Uuid,
}

/// Inserts a new resource version snapshot for audit history.
pub fn insert_version(conn: &mut PgConnection, v: &NewResourceVersion) -> QueryResult<usize> {
    diesel::insert_into(resource_versions::table)
        .values(v)
        .execute(conn)
}

#[derive(Queryable, Selectable, Debug, Clone, serde::Serialize)]
#[diesel(table_name = resource_versions)]
pub struct ResourceVersionRow {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub version_number: i32,
    pub snapshot: serde_json::Value,
    pub changed_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Lists all version snapshots for a resource, ordered by version number descending.
pub fn list_versions(conn: &mut PgConnection, resource_id: Uuid) -> QueryResult<Vec<ResourceVersionRow>> {
    resource_versions::table
        .filter(resource_versions::resource_id.eq(resource_id))
        .order(resource_versions::version_number.desc())
        .select(ResourceVersionRow::as_select())
        .load(conn)
}
