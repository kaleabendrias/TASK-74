use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::media_files;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = media_files)]
pub struct MediaFileRow {
    pub id: Uuid,
    pub original_name: String,
    pub stored_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = media_files)]
pub struct NewMediaFile<'a> {
    pub original_name: &'a str,
    pub stored_path: &'a str,
    pub mime_type: &'a str,
    pub size_bytes: i64,
    pub checksum_sha256: &'a str,
    pub uploaded_by: Uuid,
}

pub fn insert(conn: &mut PgConnection, new: &NewMediaFile) -> QueryResult<MediaFileRow> {
    diesel::insert_into(media_files::table)
        .values(new)
        .returning(MediaFileRow::as_returning())
        .get_result(conn)
}

pub fn find_by_id(conn: &mut PgConnection, id: Uuid) -> QueryResult<MediaFileRow> {
    media_files::table
        .find(id)
        .select(MediaFileRow::as_select())
        .first(conn)
}

pub fn ids_exist(conn: &mut PgConnection, ids: &[Uuid]) -> QueryResult<Vec<Uuid>> {
    media_files::table
        .filter(media_files::id.eq_any(ids))
        .select(media_files::id)
        .load(conn)
}
