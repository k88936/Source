//! Row struct mirroring the `local_assets` table, plus its media conversion.

use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use crate::media::LocalAsset;
use crate::store::schema;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::local_assets)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct LocalAssetRow {
    pub(crate) id: String,
    pub(crate) album_id: Option<String>,
    pub(crate) name: String,
    pub(crate) mime: Option<String>,
    pub(crate) size: Option<i64>,
    pub(crate) date_taken: Option<i64>,
    pub(crate) uri: String,
    pub(crate) width: Option<i32>,
    pub(crate) height: Option<i32>,
    pub(crate) duration_ms: Option<i64>,
}

impl From<LocalAssetRow> for LocalAsset {
    fn from(row: LocalAssetRow) -> Self {
        LocalAsset {
            id: row.id,
            album_id: row.album_id,
            name: row.name,
            mime: row.mime,
            size: row.size.unwrap_or(0),
            date_taken: row.date_taken.unwrap_or(0),
            uri: row.uri,
            width: row.width,
            height: row.height,
            duration_ms: row.duration_ms,
        }
    }
}
