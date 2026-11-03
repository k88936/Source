//! Row struct mirroring the `assets` table, plus its DTO conversion.

use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use super::super::folder::FolderId;
use super::super::schema;
use super::dto::{AssetId, AssetSummary};
use super::values::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::assets)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct AssetRow {
    pub(crate) id: String,
    pub(crate) folder_id: String,
    pub(crate) name: String,
    pub(crate) media_type: String,
    pub(crate) mime: Option<String>,
    pub(crate) size: i64,
    pub(crate) taken_at: i64,
    pub(crate) taken_at_source: String,
    pub(crate) width: Option<i32>,
    pub(crate) height: Option<i32>,
    pub(crate) duration_ms: Option<i64>,
    pub(crate) favorite: bool,
    pub(crate) thumb_hash: Option<String>,
}

impl From<AssetRow> for AssetSummary {
    fn from(row: AssetRow) -> Self {
        AssetSummary {
            id: AssetId(row.id),
            folder_id: FolderId(row.folder_id),
            name: row.name,
            media_type: media_type_from_str(&row.media_type),
            mime: row.mime,
            size: row.size,
            taken_at: row.taken_at,
            taken_at_source: taken_source_from_str(&row.taken_at_source),
            width: row.width,
            height: row.height,
            duration_ms: row.duration_ms,
            favorite: row.favorite,
            thumb_hash: row.thumb_hash,
        }
    }
}
