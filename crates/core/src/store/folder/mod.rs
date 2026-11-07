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

/// How many ids one delete names: a whole album can be thousands of files.
const PRUNE_CHUNK: usize = 500;

#[async_trait]
pub trait FolderStore: Send + Sync {
    async fn upsert_folder(&self, folder: &FolderRecord) -> Result<(), CatalogError>;
    async fn get_folder(&self, id: &str) -> Result<Option<FolderRecord>, CatalogError>;
    async fn get_folder_by_uri(&self, uri: &str) -> Result<Option<FolderRecord>, CatalogError>;
    async fn get_folder_state(&self, id: &str) -> Result<Option<FolderSyncState>, CatalogError>;
    async fn mark_folder(
        &self,
        id: &str,
        status: &str,
        remote_updated_at: Option<i64>,
        now: i64,
    ) -> Result<(), CatalogError>;
    /// Write one listing page, keeping the rows the album already has.
    async fn upsert_album_assets(&self, assets: &[AssetRecord]) -> Result<(), CatalogError>;
    /// Drop the assets a complete listing no longer holds, given every id it listed.
    async fn prune_album_assets(&self, folder_id: &str, listed: &[String]) -> Result<(), CatalogError>;
    /// The names of the media files the album currently holds.
    async fn list_album_asset_names(&self, folder_id: &str) -> Result<Vec<String>, CatalogError>;
}

pub(crate) async fn upsert_folder(
    cat: &SqliteCatalog,
    folder: &FolderRecord,
) -> Result<(), CatalogError> {
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

pub(crate) async fn get_folder(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<FolderRecord>, CatalogError> {
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

pub(crate) async fn get_folder_by_uri(
    cat: &SqliteCatalog,
    uri: &str,
) -> Result<Option<FolderRecord>, CatalogError> {
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

pub(crate) async fn get_folder_state(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<FolderSyncState>, CatalogError> {
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
    remote_updated_at: Option<i64>,
    now: i64,
) -> Result<(), CatalogError> {
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
        ))
        .on_conflict(sync_state::folder_id)
        .do_update()
        .set((
            sync_state::last_listed_at.eq(excluded(sync_state::last_listed_at)),
            sync_state::remote_updated_at.eq(excluded(sync_state::remote_updated_at)),
            sync_state::status.eq(excluded(sync_state::status)),
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

/// The names of an album's files as the catalog last listed them. Used to tell
/// a file that is already in the album from one that still has to go up.
pub(crate) async fn list_album_asset_names(
    cat: &SqliteCatalog,
    folder_id: &str,
) -> Result<Vec<String>, CatalogError> {
    let mut conn = cat.conn().await?;
    assets::table
        .filter(assets::folder_id.eq(folder_id))
        .select(assets::name)
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)
}

/// Write one listing page without touching the rest of the album.
///
/// Assets already in the album keep their row, so the thumbnail hash and the
/// parsed EXIF survive a refresh; a page that does not mention an asset leaves
/// it for the prune that follows the complete listing.
pub(crate) async fn upsert_album_assets(
    cat: &SqliteCatalog,
    asset_records: &[AssetRecord],
) -> Result<(), CatalogError> {
    if asset_records.is_empty() {
        return Ok(());
    }
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        for asset in asset_records {
            upsert_asset_row(conn, asset).await?;
        }
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

/// Drop the assets a complete listing no longer holds.
///
/// `listed` must be every id the listing returned: a page-by-page prune would
/// delete the rows of the pages it has not read yet.
pub(crate) async fn prune_album_assets(
    cat: &SqliteCatalog,
    folder_id: &str,
    listed: &[String],
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    let existing = assets::table
        .filter(assets::folder_id.eq(folder_id))
        .select(assets::id)
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)?;
    let keep: std::collections::HashSet<&str> = listed.iter().map(String::as_str).collect();
    let stale: Vec<String> = existing
        .into_iter()
        .filter(|id| !keep.contains(id.as_str()))
        .collect();
    for chunk in stale.chunks(PRUNE_CHUNK) {
        diesel::delete(assets::table.filter(assets::id.eq_any(chunk)))
            .execute(&mut conn)
            .await
            .map_err(db_err)?;
    }
    Ok(())
}

#[async_trait]
impl FolderStore for SqliteCatalog {
    async fn upsert_folder(&self, folder_record: &FolderRecord) -> Result<(), CatalogError> {
        upsert_folder(self, folder_record).await
    }

    async fn get_folder(&self, id: &str) -> Result<Option<FolderRecord>, CatalogError> {
        get_folder(self, id).await
    }

    async fn get_folder_by_uri(&self, uri: &str) -> Result<Option<FolderRecord>, CatalogError> {
        get_folder_by_uri(self, uri).await
    }

    async fn get_folder_state(&self, id: &str) -> Result<Option<FolderSyncState>, CatalogError> {
        get_folder_state(self, id).await
    }

    async fn mark_folder(
        &self,
        id: &str,
        status: &str,
        remote_updated_at: Option<i64>,
        now: i64,
    ) -> Result<(), CatalogError> {
        mark_folder(self, id, status, remote_updated_at, now).await
    }

    async fn upsert_album_assets(&self, asset_records: &[AssetRecord]) -> Result<(), CatalogError> {
        upsert_album_assets(self, asset_records).await
    }

    async fn prune_album_assets(&self, folder_id: &str, listed: &[String]) -> Result<(), CatalogError> {
        prune_album_assets(self, folder_id, listed).await
    }

    async fn list_album_asset_names(&self, folder_id: &str) -> Result<Vec<String>, CatalogError> {
        list_album_asset_names(self, folder_id).await
    }
}
