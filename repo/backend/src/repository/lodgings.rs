use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{lodgings, lodging_periods, lodging_rent_changes};

// ── Lodgings ──

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = lodgings)]
pub struct LodgingRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub amenities: serde_json::Value,
    pub facility_id: Option<Uuid>,
    pub deposit_amount: Option<bigdecimal::BigDecimal>,
    pub monthly_rent: Option<bigdecimal::BigDecimal>,
    pub deposit_cap_validated: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = lodgings)]
pub struct NewLodging<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub state: &'a str,
    pub amenities: serde_json::Value,
    pub facility_id: Option<Uuid>,
    pub deposit_amount: Option<bigdecimal::BigDecimal>,
    pub monthly_rent: Option<bigdecimal::BigDecimal>,
    pub deposit_cap_validated: bool,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = lodgings)]
pub struct LodgingUpdate {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub state: Option<String>,
    pub amenities: Option<serde_json::Value>,
    pub facility_id: Option<Option<Uuid>>,
    pub deposit_amount: Option<Option<bigdecimal::BigDecimal>>,
    pub monthly_rent: Option<Option<bigdecimal::BigDecimal>>,
    pub deposit_cap_validated: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub fn insert_lodging(conn: &mut PgConnection, new: &NewLodging) -> QueryResult<LodgingRow> {
    diesel::insert_into(lodgings::table)
        .values(new)
        .returning(LodgingRow::as_returning())
        .get_result(conn)
}

pub fn find_lodging_by_id(conn: &mut PgConnection, id: Uuid) -> QueryResult<LodgingRow> {
    lodgings::table
        .find(id)
        .select(LodgingRow::as_select())
        .first(conn)
}

pub fn update_lodging(
    conn: &mut PgConnection,
    id: Uuid,
    changeset: &LodgingUpdate,
) -> QueryResult<LodgingRow> {
    diesel::update(lodgings::table.find(id))
        .set(changeset)
        .returning(LodgingRow::as_returning())
        .get_result(conn)
}

pub fn list_lodgings(
    conn: &mut PgConnection,
    facility_id: Option<Uuid>,
) -> QueryResult<Vec<LodgingRow>> {
    let mut query = lodgings::table.into_boxed();
    if let Some(fid) = facility_id {
        query = query.filter(lodgings::facility_id.eq(fid));
    }
    query.order(lodgings::created_at.desc()).select(LodgingRow::as_select()).load(conn)
}

// ── Periods ──

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = lodging_periods)]
pub struct PeriodRow {
    pub id: Uuid,
    pub lodging_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_nights: i32,
    pub max_nights: i32,
    pub vacancy: bool,
}

#[derive(Insertable)]
#[diesel(table_name = lodging_periods)]
pub struct NewPeriod {
    pub lodging_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_nights: i32,
    pub max_nights: i32,
    pub vacancy: bool,
}

pub fn list_periods(conn: &mut PgConnection, lodging_id: Uuid) -> QueryResult<Vec<PeriodRow>> {
    lodging_periods::table
        .filter(lodging_periods::lodging_id.eq(lodging_id))
        .order(lodging_periods::start_date.asc())
        .select(PeriodRow::as_select())
        .load(conn)
}

pub fn insert_period(conn: &mut PgConnection, new: &NewPeriod) -> QueryResult<PeriodRow> {
    diesel::insert_into(lodging_periods::table)
        .values(new)
        .returning(PeriodRow::as_returning())
        .get_result(conn)
}

pub fn find_overlapping_periods(
    conn: &mut PgConnection,
    lodging_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
) -> QueryResult<Vec<PeriodRow>> {
    lodging_periods::table
        .filter(lodging_periods::lodging_id.eq(lodging_id))
        .filter(lodging_periods::start_date.lt(end))
        .filter(lodging_periods::end_date.gt(start))
        .select(PeriodRow::as_select())
        .load(conn)
}

// ── Rent Changes ──

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = lodging_rent_changes)]
pub struct RentChangeRow {
    pub id: Uuid,
    pub lodging_id: Uuid,
    pub proposed_rent: bigdecimal::BigDecimal,
    pub proposed_deposit: bigdecimal::BigDecimal,
    pub status: String,
    pub requested_by: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = lodging_rent_changes)]
pub struct NewRentChange {
    pub lodging_id: Uuid,
    pub proposed_rent: bigdecimal::BigDecimal,
    pub proposed_deposit: bigdecimal::BigDecimal,
    pub status: String,
    pub requested_by: Uuid,
}

pub fn insert_rent_change(
    conn: &mut PgConnection,
    new: &NewRentChange,
) -> QueryResult<RentChangeRow> {
    diesel::insert_into(lodging_rent_changes::table)
        .values(new)
        .returning(RentChangeRow::as_returning())
        .get_result(conn)
}

pub fn find_rent_change(
    conn: &mut PgConnection,
    id: Uuid,
) -> QueryResult<RentChangeRow> {
    lodging_rent_changes::table
        .find(id)
        .select(RentChangeRow::as_select())
        .first(conn)
}

pub fn update_rent_change_status(
    conn: &mut PgConnection,
    id: Uuid,
    new_status: &str,
    reviewer: Uuid,
) -> QueryResult<RentChangeRow> {
    diesel::update(lodging_rent_changes::table.find(id))
        .set((
            lodging_rent_changes::status.eq(new_status),
            lodging_rent_changes::reviewed_by.eq(Some(reviewer)),
            lodging_rent_changes::reviewed_at.eq(Some(Utc::now())),
        ))
        .returning(RentChangeRow::as_returning())
        .get_result(conn)
}
