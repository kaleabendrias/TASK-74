use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::config_parameters;

#[derive(Queryable, Selectable, Debug, serde::Serialize)]
#[diesel(table_name = config_parameters)]
pub struct ConfigParamRow {
    pub id: Uuid,
    pub profile: String,
    pub key: String,
    pub value: String,
    pub feature_switch: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = config_parameters)]
struct NewConfigParam<'a> {
    pub profile: &'a str,
    pub key: &'a str,
    pub value: &'a str,
    pub feature_switch: bool,
}

/// Lists all config parameters for a given profile.
pub fn list_by_profile(conn: &mut PgConnection, profile: &str) -> QueryResult<Vec<ConfigParamRow>> {
    config_parameters::table
        .filter(config_parameters::profile.eq(profile))
        .order(config_parameters::key.asc())
        .select(ConfigParamRow::as_select())
        .load(conn)
}

/// Upserts a config parameter: inserts if new, updates value if key exists.
pub fn upsert(
    conn: &mut PgConnection,
    profile: &str,
    key: &str,
    value: &str,
    feature_switch: bool,
) -> QueryResult<ConfigParamRow> {
    // Try update first
    let updated = diesel::update(
        config_parameters::table
            .filter(config_parameters::profile.eq(profile))
            .filter(config_parameters::key.eq(key)),
    )
    .set((
        config_parameters::value.eq(value),
        config_parameters::feature_switch.eq(feature_switch),
        config_parameters::updated_at.eq(Utc::now()),
    ))
    .returning(ConfigParamRow::as_returning())
    .get_results(conn)?;

    if let Some(row) = updated.into_iter().next() {
        return Ok(row);
    }

    // Insert new
    diesel::insert_into(config_parameters::table)
        .values(&NewConfigParam { profile, key, value, feature_switch })
        .returning(ConfigParamRow::as_returning())
        .get_result(conn)
}
