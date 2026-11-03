//! The library root and the remote folders mirrored from the server: the
//! contract and the diesel statements behind it.

use std::collections::HashSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::dto::*;
use super::query::*;
use super::rows::*;
use super::schema::assets;
use super::schema::folders;
use super::schema::sync_state;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FolderRef {
    pub id: FolderId,
    pub uri: String,
    pub name: String,
    /// Path relative to the user's root (`/Photos/2024` style).
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct FolderRecord {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub uri: String,
    pub path: String,
    pub updated_at: i64,
    pub is_album: bool,
    pub cover_asset_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderSyncState {
    pub folder_id: String,
    pub status: String,
    pub remote_updated_at: Option<i64>,
    pub last_listed_at: Option<i64>,
    pub last_error: Option<String>,
}

impl FolderSyncState {
    pub fn is_done(&self) -> bool {
        self.status == "done"
    }
}

#[async_trait]
pub trait FolderStore: Send + Sync {
    async fn set_library_root(&self, root: &FolderRef) -> Result<(), AppError>;
    async fn library_root(&self) -> Result<Option<FolderRef>, AppError>;

    async fn upsert_folder(&self, folder: &FolderRecord) -> Result<(), AppError>;
    async fn folder(&self, id: &str) -> Result<Option<FolderRecord>, AppError>;
    async fn folder_by_uri(&self, uri: &str) -> Result<Option<FolderRecord>, AppError>;
    async fn folder_state(&self, id: &str) -> Result<Option<FolderSyncState>, AppError>;
    async fn mark_folder(
        &self,
        id: &str,
        status: &str,
        error: Option<&str>,
        remote_updated_at: Option<i64>,
        now: i64,
    ) -> Result<(), AppError>;
    async fn replace_folder_assets(
        &self,
        folder_id: &str,
        assets: &[AssetRecord],
        now: i64,
    ) -> Result<(), AppError>;
    /// Drop every folder outside `keep`, with its assets and sync state.
    ///
    /// Only a completed full pass has listed the whole tree, so a caller must
    /// not prune after a partial or incremental run.
    async fn retain_folders(&self, keep: &HashSet<String>) -> Result<(), AppError>;
}

pub(crate) async fn set_library_root(
    cat: &SqliteCatalog,
    root: &FolderRef,
) -> Result<(), AppError> {
    let value = serde_json::to_string(root)
        .map_err(|e| AppError::internal(format!("serialize root: {e}")))?;
    cat.set_setting("library_root", &value).await
}

pub(crate) async fn library_root(cat: &SqliteCatalog) -> Result<Option<FolderRef>, AppError> {
    match cat.get_setting("library_root").await? {
        Some(raw) => serde_json::from_str(&raw)
            .map(Some)
            .map_err(|e| AppError::internal(format!("parse library root: {e}"))),
        None => Ok(None),
    }
}

pub(crate) async fn upsert_folder(
    cat: &SqliteCatalog,
    folder: &FolderRecord,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    // Curation fields are owned by the UI: only replace them when the
    // incoming record actually carries a value.
    let existing = folders::table
        .filter(folders::id.eq(folder.id.clone()))
        .select((folders::cover_asset_id, folders::description))
        .first::<(Option<String>, Option<String>)>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    let (cover_asset_id, description) = match existing {
        Some((cover, description)) => (
            folder.cover_asset_id.clone().or(cover),
            folder.description.clone().or(description),
        ),
        None => (folder.cover_asset_id.clone(), folder.description.clone()),
    };

    diesel::insert_into(folders::table)
        .values((
            folders::id.eq(folder.id.clone()),
            folders::parent_id.eq(folder.parent_id.clone()),
            folders::name.eq(folder.name.clone()),
            folders::uri.eq(folder.uri.clone()),
            folders::path.eq(folder.path.clone()),
            folders::updated_at.eq(folder.updated_at),
            folders::is_album.eq(false),
            folders::cover_asset_id.eq(cover_asset_id),
            folders::description.eq(description),
        ))
        .on_conflict(folders::id)
        .do_update()
        .set((
            folders::parent_id.eq(excluded(folders::parent_id)),
            folders::name.eq(excluded(folders::name)),
            folders::uri.eq(excluded(folders::uri)),
            folders::path.eq(excluded(folders::path)),
            folders::updated_at.eq(excluded(folders::updated_at)),
            folders::cover_asset_id.eq(excluded(folders::cover_asset_id)),
            folders::description.eq(excluded(folders::description)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn folder(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<FolderRecord>, AppError> {
    let mut conn = cat.conn().await?;
    folders::table
        .filter(folders::id.eq(id))
        .select(FolderRow::as_select())
        .first::<FolderRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(FolderRecord::from))
        .map_err(db_err)
}

pub(crate) async fn folder_by_uri(
    cat: &SqliteCatalog,
    uri: &str,
) -> Result<Option<FolderRecord>, AppError> {
    let mut conn = cat.conn().await?;
    // Several folders can share a URI after a remote rename; the deepest path
    // wins. Diesel has no `length()` helper and the candidate set is tiny.
    let rows = folders::table
        .filter(folders::uri.eq(uri))
        .select(FolderRow::as_select())
        .load::<FolderRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(rows
        .into_iter()
        .max_by_key(|folder| folder.path.len())
        .map(FolderRecord::from))
}

pub(crate) async fn folder_state(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<FolderSyncState>, AppError> {
    let mut conn = cat.conn().await?;
    sync_state::table
        .filter(sync_state::folder_id.eq(id))
        .select(SyncStateRow::as_select())
        .first::<SyncStateRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(FolderSyncState::from))
        .map_err(db_err)
}

pub(crate) async fn mark_folder(
    cat: &SqliteCatalog,
    id: &str,
    status: &str,
    error: Option<&str>,
    remote_updated_at: Option<i64>,
    now: i64,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    let previous = sync_state::table
        .filter(sync_state::folder_id.eq(id))
        .select(sync_state::remote_updated_at)
        .first::<Option<i64>>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?
        .flatten();
    let remote_updated_at = remote_updated_at.or(previous);

    diesel::insert_into(sync_state::table)
        .values((
            sync_state::folder_id.eq(id),
            sync_state::last_listed_at.eq(now),
            sync_state::remote_updated_at.eq(remote_updated_at),
            sync_state::status.eq(status),
            sync_state::last_error.eq(error),
        ))
        .on_conflict(sync_state::folder_id)
        .do_update()
        .set((
            sync_state::last_listed_at.eq(excluded(sync_state::last_listed_at)),
            sync_state::remote_updated_at.eq(excluded(sync_state::remote_updated_at)),
            sync_state::status.eq(excluded(sync_state::status)),
            sync_state::last_error.eq(excluded(sync_state::last_error)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;

    diesel::update(folders::table.filter(folders::id.eq(id)))
        .set(folders::indexed_at.eq(Some(now)))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

/// Drop every folder outside `keep`, with its assets and sync state.
///
/// The assets go through the `ON DELETE CASCADE` foreign key, so a remote
/// deletion cannot leave a timeline entry pointing at a folder that no longer
/// exists in the tree.
pub(crate) async fn retain_folders(
    cat: &SqliteCatalog,
    keep: &HashSet<String>,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    let known = folders::table
        .select(folders::id)
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)?;
    let stale: Vec<String> = known.into_iter().filter(|id| !keep.contains(id)).collect();
    if stale.is_empty() {
        return Ok(());
    }
    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::delete(sync_state::table.filter(sync_state::folder_id.eq_any(&stale)))
            .execute(&mut *conn)
            .await?;
        diesel::delete(folders::table.filter(folders::id.eq_any(&stale)))
            .execute(&mut *conn)
            .await?;
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

pub(crate) async fn replace_folder_assets(
    cat: &SqliteCatalog,
    folder_id: &str,
    asset_records: &[AssetRecord],
    _now: i64,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::delete(assets::table.filter(assets::folder_id.eq(folder_id)))
            .execute(&mut *conn)
            .await?;
        for asset in asset_records {
            upsert_asset_row(conn, asset).await?;
        }
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

#[async_trait]
impl FolderStore for SqliteCatalog {
    async fn set_library_root(&self, root: &FolderRef) -> Result<(), AppError> {
        set_library_root(self, root).await
    }

    async fn library_root(&self) -> Result<Option<FolderRef>, AppError> {
        library_root(self).await
    }

    async fn upsert_folder(&self, folder_record: &FolderRecord) -> Result<(), AppError> {
        upsert_folder(self, folder_record).await
    }

    async fn folder(&self, id: &str) -> Result<Option<FolderRecord>, AppError> {
        folder(self, id).await
    }

    async fn folder_by_uri(&self, uri: &str) -> Result<Option<FolderRecord>, AppError> {
        folder_by_uri(self, uri).await
    }

    async fn folder_state(&self, id: &str) -> Result<Option<FolderSyncState>, AppError> {
        folder_state(self, id).await
    }

    async fn mark_folder(
        &self,
        id: &str,
        status: &str,
        error: Option<&str>,
        remote_updated_at: Option<i64>,
        now: i64,
    ) -> Result<(), AppError> {
        mark_folder(self, id, status, error, remote_updated_at, now).await
    }

    async fn replace_folder_assets(
        &self,
        folder_id: &str,
        asset_records: &[AssetRecord],
        now: i64,
    ) -> Result<(), AppError> {
        replace_folder_assets(self, folder_id, asset_records, now).await
    }

    async fn retain_folders(&self, keep: &HashSet<String>) -> Result<(), AppError> {
        retain_folders(self, keep).await
    }
}
