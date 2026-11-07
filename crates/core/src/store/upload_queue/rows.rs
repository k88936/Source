//! Row struct mirroring the `upload_queue` table, plus its DTO conversion.

use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use super::dto::{UploadTask, UploadTaskId};
use super::schema;
use super::values::upload_state_from_str;
use crate::store::LocalAssetId;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::upload_queue)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct UploadRow {
    pub(crate) id: String,
    pub(crate) local_asset_id: String,
    pub(crate) dest_uri: String,
    pub(crate) dest_name: Option<String>,
    pub(crate) name: String,
    pub(crate) state: String,
    pub(crate) uploaded: i64,
    pub(crate) total: i64,
    pub(crate) speed: i64,
    pub(crate) eta_seconds: Option<i64>,
    pub(crate) attempts: i64,
    pub(crate) error: Option<String>,
}

impl From<UploadRow> for UploadTask {
    fn from(row: UploadRow) -> Self {
        UploadTask {
            id: UploadTaskId(row.id),
            local_asset_id: LocalAssetId(row.local_asset_id),
            dest_uri: row.dest_uri,
            dest_name: row.dest_name,
            name: row.name,
            state: upload_state_from_str(&row.state),
            uploaded: row.uploaded,
            total: row.total,
            speed: row.speed,
            eta_seconds: row.eta_seconds.map(|value| value.max(0) as u32),
            attempts: row.attempts.max(0) as u32,
            error: row.error,
        }
    }
}
