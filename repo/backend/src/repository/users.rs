use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::users;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub facility_id: Option<Uuid>,
    pub totp_secret: Option<Vec<u8>>,
    pub mfa_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub role: &'a str,
    pub facility_id: Option<Uuid>,
    pub mfa_enabled: bool,
}

/// Finds a user by username in the database.
pub fn find_by_username(conn: &mut PgConnection, uname: &str) -> QueryResult<UserRow> {
    users::table
        .filter(users::username.eq(uname))
        .select(UserRow::as_select())
        .first(conn)
}

/// Finds a user by their unique ID.
pub fn find_by_id(conn: &mut PgConnection, uid: Uuid) -> QueryResult<UserRow> {
    users::table
        .find(uid)
        .select(UserRow::as_select())
        .first(conn)
}

/// Inserts a new user into the database and returns the created row.
pub fn insert(conn: &mut PgConnection, new: &NewUser) -> QueryResult<UserRow> {
    diesel::insert_into(users::table)
        .values(new)
        .returning(UserRow::as_returning())
        .get_result(conn)
}
