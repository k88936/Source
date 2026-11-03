//! Row structs mirroring `schema.rs`, plus their DTO conversions.

use super::values::*;
use super::*;

// ---------------------------------------------------------------------------
// Rows

#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::assets)]
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::folders)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct FolderRow {
    pub(crate) id: String,
    pub(crate) parent_id: Option<String>,
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) path: String,
    pub(crate) updated_at: i64,
    pub(crate) is_album: bool,
    pub(crate) cover_asset_id: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::sync_state)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct SyncStateRow {
    pub(crate) folder_id: String,
    pub(crate) status: String,
    pub(crate) remote_updated_at: Option<i64>,
    pub(crate) last_listed_at: Option<i64>,
    pub(crate) last_error: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::local_assets)]
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::upload_queue)]
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

impl From<FolderRow> for FolderRecord {
    fn from(row: FolderRow) -> Self {
        FolderRecord {
            id: row.id,
            parent_id: row.parent_id,
            name: row.name,
            uri: row.uri,
            path: row.path,
            updated_at: row.updated_at,
            is_album: row.is_album,
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

impl From<UploadRow> for UploadTask {
    fn from(row: UploadRow) -> Self {
        UploadTask {
            id: UploadTaskId(row.id),
            local_asset_id: row.local_asset_id,
            dest_uri: row.dest_uri,
            dest_name: row.dest_name,
            name: row.name,
            state: upload_state_from_str(&row.state),
            uploaded: row.uploaded,
            total: row.total,
            speed: row.speed,
            eta_seconds: row.eta_seconds.map(|value| value.max(0) as u32),
            attempts: row.attempts.max(0) as u32,
            error: row.error.and_then(|raw| serde_json::from_str(&raw).ok()),
        }
    }
}
