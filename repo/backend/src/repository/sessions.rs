use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{sessions, csrf_tokens};

// ── Sessions ──

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = sessions)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    pub user_id: Uuid,
    pub token_hash: &'a str,
    pub expires_at: DateTime<Utc>,
}

/// Creates a new session record in the database.
pub fn create_session(conn: &mut PgConnection, new: &NewSession) -> QueryResult<SessionRow> {
    diesel::insert_into(sessions::table)
        .values(new)
        .returning(SessionRow::as_returning())
        .get_result(conn)
}

/// Finds a non-expired session by its token hash.
pub fn find_session_by_token_hash(conn: &mut PgConnection, hash: &str) -> QueryResult<SessionRow> {
    sessions::table
        .filter(sessions::token_hash.eq(hash))
        .filter(sessions::expires_at.gt(Utc::now()))
        .select(SessionRow::as_select())
        .first(conn)
}

/// Deletes a session by its primary key ID.
pub fn delete_session(conn: &mut PgConnection, session_id: Uuid) -> QueryResult<usize> {
    diesel::delete(sessions::table.find(session_id)).execute(conn)
}

/// Deletes a session matching the given token hash.
pub fn delete_session_by_token_hash(conn: &mut PgConnection, hash: &str) -> QueryResult<usize> {
    diesel::delete(sessions::table.filter(sessions::token_hash.eq(hash))).execute(conn)
}

/// Counts the number of non-expired active sessions.
pub fn count_active_sessions(conn: &mut PgConnection) -> QueryResult<i64> {
    sessions::table
        .filter(sessions::expires_at.gt(Utc::now()))
        .count()
        .get_result(conn)
}

// ── CSRF Tokens ──

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = csrf_tokens)]
pub struct CsrfTokenRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = csrf_tokens)]
pub struct NewCsrfToken<'a> {
    pub session_id: Uuid,
    pub token_hash: &'a str,
    pub expires_at: DateTime<Utc>,
}

/// Creates a new CSRF token associated with a session.
pub fn create_csrf_token(conn: &mut PgConnection, new: &NewCsrfToken) -> QueryResult<CsrfTokenRow> {
    diesel::insert_into(csrf_tokens::table)
        .values(new)
        .returning(CsrfTokenRow::as_returning())
        .get_result(conn)
}

/// Finds a non-expired CSRF token by its hash, bound to a specific session.
pub fn find_csrf_token(conn: &mut PgConnection, hash: &str, session_id: Uuid) -> QueryResult<CsrfTokenRow> {
    csrf_tokens::table
        .filter(csrf_tokens::token_hash.eq(hash))
        .filter(csrf_tokens::session_id.eq(session_id))
        .filter(csrf_tokens::expires_at.gt(Utc::now()))
        .select(CsrfTokenRow::as_select())
        .first(conn)
}
