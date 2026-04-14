use chrono::Utc;
use diesel::Connection;
use diesel::PgConnection;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::model::*;
use crate::repository::inventory as repo;

/// Creates a new inventory lot after validating item name and quantity.
pub fn create_lot(
    conn: &mut PgConnection,
    req: &CreateLotRequest,
) -> Result<LotResponse, ApiError> {
    if req.item_name.is_empty() {
        return Err(ApiError::unprocessable(
            "VALIDATION_ERROR",
            "item_name is required",
        ));
    }
    if req.quantity_on_hand < 0 {
        return Err(ApiError::unprocessable(
            "VALIDATION_ERROR",
            "quantity_on_hand must be non-negative",
        ));
    }

    let new = repo::NewLot {
        facility_id: req.facility_id,
        warehouse_id: req.warehouse_id,
        bin_id: req.bin_id,
        item_name: req.item_name.clone(),
        lot_number: req.lot_number.clone(),
        quantity_on_hand: req.quantity_on_hand,
        quantity_reserved: 0,
        expiration_date: req.expiration_date,
    };

    let row = repo::insert_lot(conn, &new)?;
    Ok(lot_to_response(&row))
}

/// Retrieves a single inventory lot by its ID.
pub fn get_lot(conn: &mut PgConnection, id: Uuid) -> Result<LotResponse, ApiError> {
    let row = repo::find_lot_by_id(conn, id)?;
    Ok(lot_to_response(&row))
}

/// Lists inventory lots, optionally filtered by facility and near-expiry status.
pub fn list_lots(
    conn: &mut PgConnection,
    facility_id: Option<Uuid>,
    near_expiry: bool,
) -> Result<Vec<LotResponse>, ApiError> {
    let rows = repo::list_lots(conn, facility_id, near_expiry)?;
    Ok(rows.iter().map(lot_to_response).collect())
}

/// Reserves a quantity from a lot and records an immutable outbound transaction.
pub fn reserve(
    conn: &mut PgConnection,
    lot_id: Uuid,
    quantity: i32,
    user_id: Uuid,
) -> Result<LotResponse, ApiError> {
    if quantity <= 0 {
        return Err(ApiError::unprocessable(
            "VALIDATION_ERROR",
            "Reservation quantity must be positive",
        ));
    }

    conn.transaction(|conn| {
        let row = repo::reserve_quantity(conn, lot_id, quantity)?;
        repo::insert_transaction(conn, &repo::NewTransaction {
            lot_id,
            direction: "outbound".to_string(),
            quantity,
            reason: Some("reservation".to_string()),
            performed_by: user_id,
            is_immutable: true,
        })?;
        Ok(row)
    }).map(|row| lot_to_response(&row))
    .map_err(|e| match e {
        diesel::result::Error::NotFound => ApiError::conflict(
            "Insufficient quantity on hand for reservation",
        ),
        _ => ApiError::from(e),
    })
}

/// Creates an inbound or outbound inventory transaction, checking quantity sufficiency for outbound.
pub fn create_transaction(
    conn: &mut PgConnection,
    req: &CreateTransactionRequest,
    user_id: Uuid,
) -> Result<TransactionResponse, ApiError> {
    if req.quantity <= 0 {
        return Err(ApiError::unprocessable(
            "VALIDATION_ERROR",
            "Transaction quantity must be positive",
        ));
    }
    if req.direction != "inbound" && req.direction != "outbound" {
        return Err(ApiError::unprocessable(
            "VALIDATION_ERROR",
            "Direction must be 'inbound' or 'outbound'",
        ));
    }

    conn.transaction(|conn| {
        // Update lot quantity
        let lot = repo::find_lot_by_id(conn, req.lot_id)?;

        if req.direction == "outbound" && lot.quantity_on_hand < req.quantity {
            return Err(diesel::result::Error::RollbackTransaction);
        }

        if req.direction == "outbound" {
            repo::update_lot_quantity(conn, req.lot_id, -req.quantity)?;
        } else {
            repo::update_lot_quantity(conn, req.lot_id, req.quantity)?;
        }

        // Insert transaction record
        let new = repo::NewTransaction {
            lot_id: req.lot_id,
            direction: req.direction.clone(),
            quantity: req.quantity,
            reason: req.reason.clone(),
            performed_by: user_id,
            is_immutable: true,
        };
        let row = repo::insert_transaction(conn, &new)?;
        Ok(row)
    }).map(|row| tx_to_response(&row))
    .map_err(|e| match e {
        diesel::result::Error::RollbackTransaction => ApiError::conflict(
            "Insufficient quantity on hand for outbound transaction",
        ),
        _ => ApiError::from(e),
    })
}

/// Lists inventory transactions matching the given filter criteria.
pub fn list_transactions(
    conn: &mut PgConnection,
    query: &TransactionQuery,
) -> Result<Vec<TransactionResponse>, ApiError> {
    let filter = repo::TransactionFilter {
        lot_id: query.lot_id,
        direction: query.direction.clone(),
        performed_by: query.performed_by,
        from_date: query.from_date,
        to_date: query.to_date,
    };
    let rows = repo::list_transactions(conn, &filter)?;
    Ok(rows.iter().map(tx_to_response).collect())
}

/// Lists transactions filtered to a specific set of lot IDs and additional query criteria.
pub fn list_transactions_for_lots(
    conn: &mut PgConnection,
    lot_ids: &[Uuid],
    query: &TransactionQuery,
) -> Result<Vec<TransactionResponse>, ApiError> {
    let filter = repo::TransactionFilter {
        lot_id: query.lot_id,
        direction: query.direction.clone(),
        performed_by: query.performed_by,
        from_date: query.from_date,
        to_date: query.to_date,
    };
    let rows = repo::list_transactions_for_lots(conn, lot_ids, &filter)?;
    Ok(rows.iter().map(tx_to_response).collect())
}

/// Generates an HTML audit trail report for all transactions on a given lot.
pub fn audit_print_html(
    conn: &mut PgConnection,
    lot_id: Uuid,
) -> Result<String, ApiError> {
    let lot = repo::find_lot_by_id(conn, lot_id)?;
    let txns = repo::transactions_with_usernames(conn, lot_id)?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let mut html = format!(
        r#"<!DOCTYPE html>
<html><head><title>Audit Trail — Lot {}</title>
<style>
body {{ font-family: monospace; margin: 2em; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border: 1px solid #333; padding: 6px 10px; text-align: left; }}
th {{ background: #eee; }}
.watermark {{ color: #999; font-size: 0.8em; text-align: center; margin-top: 2em; }}
</style></head><body>
<h1>Inventory Audit Trail</h1>
<p><strong>Lot:</strong> {} — {}</p>
<p><strong>On Hand:</strong> {} | <strong>Reserved:</strong> {}</p>
<table>
<tr><th>Date</th><th>Direction</th><th>Qty</th><th>Reason</th><th>Performed By</th></tr>"#,
        lot.lot_number,
        lot.lot_number,
        lot.item_name,
        lot.quantity_on_hand,
        lot.quantity_reserved
    );

    for (tx, username) in &txns {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            tx.created_at.format("%Y-%m-%d %H:%M:%S"),
            tx.direction,
            tx.quantity,
            tx.reason.as_deref().unwrap_or("—"),
            username
        ));
    }

    html.push_str(&format!(
        "</table>\n<p class=\"watermark\">Generated {} — {} transactions</p>\n</body></html>",
        now,
        txns.len()
    ));

    Ok(html)
}

// ── Helpers ──

fn lot_to_response(row: &repo::LotRow) -> LotResponse {
    let near_expiry = row.expiration_date.map_or(false, |d| {
        let cutoff = Utc::now().date_naive() + chrono::Duration::days(30);
        d <= cutoff
    });
    LotResponse {
        id: row.id,
        facility_id: row.facility_id,
        warehouse_id: row.warehouse_id,
        bin_id: row.bin_id,
        item_name: row.item_name.clone(),
        lot_number: row.lot_number.clone(),
        quantity_on_hand: row.quantity_on_hand,
        quantity_reserved: row.quantity_reserved,
        expiration_date: row.expiration_date,
        near_expiry,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn tx_to_response(row: &repo::TransactionRow) -> TransactionResponse {
    TransactionResponse {
        id: row.id,
        lot_id: row.lot_id,
        direction: row.direction.clone(),
        quantity: row.quantity,
        reason: row.reason.clone(),
        performed_by: row.performed_by,
        created_at: row.created_at,
        is_immutable: row.is_immutable,
    }
}
