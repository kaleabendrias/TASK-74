use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{inventory_lots, inventory_transactions};

// ── Lots ──

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = inventory_lots)]
pub struct LotRow {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub warehouse_id: Uuid,
    pub bin_id: Uuid,
    pub item_name: String,
    pub lot_number: String,
    pub quantity_on_hand: i32,
    pub quantity_reserved: i32,
    pub expiration_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = inventory_lots)]
pub struct NewLot {
    pub facility_id: Uuid,
    pub warehouse_id: Uuid,
    pub bin_id: Uuid,
    pub item_name: String,
    pub lot_number: String,
    pub quantity_on_hand: i32,
    pub quantity_reserved: i32,
    pub expiration_date: Option<NaiveDate>,
}

/// Inserts a new inventory lot into the database.
pub fn insert_lot(conn: &mut PgConnection, new: &NewLot) -> QueryResult<LotRow> {
    diesel::insert_into(inventory_lots::table)
        .values(new)
        .returning(LotRow::as_returning())
        .get_result(conn)
}

/// Finds an inventory lot by its unique ID.
pub fn find_lot_by_id(conn: &mut PgConnection, id: Uuid) -> QueryResult<LotRow> {
    inventory_lots::table
        .find(id)
        .select(LotRow::as_select())
        .first(conn)
}

/// Lists inventory lots, optionally filtered by facility and near-expiry status.
pub fn list_lots(
    conn: &mut PgConnection,
    facility_id: Option<Uuid>,
    near_expiry: bool,
) -> QueryResult<Vec<LotRow>> {
    let mut query = inventory_lots::table.into_boxed();
    if let Some(fid) = facility_id {
        query = query.filter(inventory_lots::facility_id.eq(fid));
    }
    if near_expiry {
        let cutoff = Utc::now().date_naive() + chrono::Duration::days(30);
        query = query.filter(inventory_lots::expiration_date.le(cutoff));
        query = query.filter(inventory_lots::expiration_date.is_not_null());
    }
    query.order(inventory_lots::created_at.desc()).select(LotRow::as_select()).load(conn)
}

/// Atomically reserves a quantity from a lot's on-hand stock.
pub fn reserve_quantity(
    conn: &mut PgConnection,
    lot_id: Uuid,
    qty: i32,
) -> QueryResult<LotRow> {
    // Atomic: decrement on_hand, increment reserved
    diesel::update(
        inventory_lots::table
            .find(lot_id)
            .filter(inventory_lots::quantity_on_hand.ge(qty)),
    )
    .set((
        inventory_lots::quantity_on_hand.eq(inventory_lots::quantity_on_hand - qty),
        inventory_lots::quantity_reserved.eq(inventory_lots::quantity_reserved + qty),
        inventory_lots::updated_at.eq(Utc::now()),
    ))
    .returning(LotRow::as_returning())
    .get_result(conn)
}

// ── Transactions ──

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = inventory_transactions)]
pub struct TransactionRow {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub direction: String,
    pub quantity: i32,
    pub reason: Option<String>,
    pub performed_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_immutable: bool,
}

#[derive(Insertable)]
#[diesel(table_name = inventory_transactions)]
pub struct NewTransaction {
    pub lot_id: Uuid,
    pub direction: String,
    pub quantity: i32,
    pub reason: Option<String>,
    pub performed_by: Uuid,
    pub is_immutable: bool,
}

/// Adjusts a lot's quantity_on_hand by the given delta (positive for inbound, negative for outbound).
pub fn update_lot_quantity(conn: &mut PgConnection, lot_id: Uuid, delta: i32) -> QueryResult<LotRow> {
    diesel::update(inventory_lots::table.find(lot_id))
        .set((
            inventory_lots::quantity_on_hand.eq(inventory_lots::quantity_on_hand + delta),
            inventory_lots::updated_at.eq(Utc::now()),
        ))
        .returning(LotRow::as_returning())
        .get_result(conn)
}

/// Inserts a new inventory transaction record.
pub fn insert_transaction(
    conn: &mut PgConnection,
    new: &NewTransaction,
) -> QueryResult<TransactionRow> {
    diesel::insert_into(inventory_transactions::table)
        .values(new)
        .returning(TransactionRow::as_returning())
        .get_result(conn)
}

pub struct TransactionFilter {
    pub lot_id: Option<Uuid>,
    pub direction: Option<String>,
    pub performed_by: Option<Uuid>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

/// Lists inventory transactions matching the given filter criteria.
pub fn list_transactions(
    conn: &mut PgConnection,
    filter: &TransactionFilter,
) -> QueryResult<Vec<TransactionRow>> {
    let mut query = inventory_transactions::table.into_boxed();

    if let Some(lid) = filter.lot_id {
        query = query.filter(inventory_transactions::lot_id.eq(lid));
    }
    if let Some(ref d) = filter.direction {
        query = query.filter(inventory_transactions::direction.eq(d));
    }
    if let Some(uid) = filter.performed_by {
        query = query.filter(inventory_transactions::performed_by.eq(uid));
    }
    if let Some(from) = filter.from_date {
        let dt = from.and_hms_opt(0, 0, 0).unwrap().and_utc();
        query = query.filter(inventory_transactions::created_at.ge(dt));
    }
    if let Some(to) = filter.to_date {
        let dt = to.and_hms_opt(23, 59, 59).unwrap().and_utc();
        query = query.filter(inventory_transactions::created_at.le(dt));
    }

    query
        .order(inventory_transactions::created_at.desc())
        .select(TransactionRow::as_select())
        .load(conn)
}

/// Lists transactions filtered to a specific set of lot IDs.
pub fn list_transactions_for_lots(
    conn: &mut PgConnection,
    lot_ids: &[Uuid],
    filter: &TransactionFilter,
) -> QueryResult<Vec<TransactionRow>> {
    let mut query = inventory_transactions::table
        .filter(inventory_transactions::lot_id.eq_any(lot_ids))
        .into_boxed();

    if let Some(lid) = filter.lot_id {
        query = query.filter(inventory_transactions::lot_id.eq(lid));
    }
    if let Some(ref d) = filter.direction {
        query = query.filter(inventory_transactions::direction.eq(d));
    }
    if let Some(uid) = filter.performed_by {
        query = query.filter(inventory_transactions::performed_by.eq(uid));
    }
    if let Some(from) = filter.from_date {
        let dt = from.and_hms_opt(0, 0, 0).unwrap().and_utc();
        query = query.filter(inventory_transactions::created_at.ge(dt));
    }
    if let Some(to) = filter.to_date {
        let dt = to.and_hms_opt(23, 59, 59).unwrap().and_utc();
        query = query.filter(inventory_transactions::created_at.le(dt));
    }

    query.order(inventory_transactions::created_at.desc())
        .select(TransactionRow::as_select())
        .load(conn)
}

/// Returns transactions for a lot joined with the performing user's username.
pub fn transactions_with_usernames(
    conn: &mut PgConnection,
    lot_id: Uuid,
) -> QueryResult<Vec<(TransactionRow, String)>> {
    use crate::schema::users;

    inventory_transactions::table
        .inner_join(users::table.on(inventory_transactions::performed_by.eq(users::id)))
        .filter(inventory_transactions::lot_id.eq(lot_id))
        .order(inventory_transactions::created_at.asc())
        .select((TransactionRow::as_select(), users::username))
        .load(conn)
}
