//! The albums a user added: the contract and the diesel statements behind it.

use async_trait::async_trait;

use self::schema::{folders, sync_state};
use super::asset::schema::assets;
use super::query::*;
use super::*;

mod dto;
pub(crate) mod rows;
pub(crate) mod schema;

pub use dto::*;

use self::rows::*;

#[async_trait]
pub trait FolderStore: Send + Sync {
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
    ) -> Result<(), AppError>;
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
            folders::name.eq(folder.name.clone()),
            folders::uri.eq(folder.uri.clone()),
            folders::path.eq(folder.path.clone()),
            folders::updated_at.eq(folder.updated_at),
            folders::cover_asset_id.eq(cover_asset_id),
            folders::description.eq(description),
        ))
        .on_conflict(folders::id)
        .do_update()
        .set((
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

/// Replace an album's assets with a fresh one-level listing.
///
/// Assets still present keep their row, so the thumbnail hash and the parsed
/// EXIF survive a refresh; only the ones missing from the listing are deleted.
pub(crate) async fn replace_folder_assets(
    cat: &SqliteCatalog,
    folder_id: &str,
    asset_records: &[AssetRecord],
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        let keep = asset_records
            .iter()
            .map(|row| row.summary.id.0.clone())
            .collect::<Vec<_>>();
        let stale = if keep.is_empty() {
            assets::table
                .filter(assets::folder_id.eq(folder_id))
                .select(assets::id)
                .load::<String>(&mut *conn)
                .await?
        } else {
            assets::table
                .filter(assets::folder_id.eq(folder_id))
                .filter(diesel::dsl::not(assets::id.eq_any(&keep)))
                .select(assets::id)
                .load::<String>(&mut *conn)
                .await?
        };
        if !stale.is_empty() {
            diesel::delete(assets::table.filter(assets::id.eq_any(&stale)))
                .execute(&mut *conn)
                .await?;
        }
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
    ) -> Result<(), AppError> {
        replace_folder_assets(self, folder_id, asset_records).await
    }
}
