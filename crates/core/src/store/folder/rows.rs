//! Row structs mirroring the `folders` and `sync_state` tables, plus their
//! DTO conversions.

use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use super::dto::{FolderRecord, FolderSyncState};
use super::schema;
use crate::store::display_path;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::folders)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct FolderRow {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) path: String,
    pub(crate) updated_at: i64,
    pub(crate) cover_asset_id: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::sync_state)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct SyncStateRow {
    pub(crate) folder_id: String,
    pub(crate) status: String,
    pub(crate) remote_updated_at: Option<i64>,
    pub(crate) last_listed_at: Option<i64>,
    pub(crate) last_error: Option<String>,
}

impl From<FolderRow> for FolderRecord {
    fn from(row: FolderRow) -> Self {
        FolderRecord {
            id: row.id,
            name: row.name,
            uri: row.uri,
            path: display_path(&row.path),
            updated_at: row.updated_at,
            cover_asset_id: row.cover_asset_id,
            description: row.description,
        }
    }
}

impl From<SyncStateRow> for FolderSyncState {
    fn from(row: SyncStateRow) -> Self {
        FolderSyncState {
            folder_id: row.folder_id,
            status: row.status,
            remote_updated_at: row.remote_updated_at,
            last_listed_at: row.last_listed_at,
            last_error: row.last_error,
        }
    }
}
