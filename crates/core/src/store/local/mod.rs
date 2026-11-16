//! The local (on-device) media library: the contract and the diesel statements
//! behind it.
//!
//! Registration and mapping are written together: a local album exists only
//! because it syncs somewhere, so there is never a directory without a target.

use async_trait::async_trait;

use crate::time::now_ms;

use self::rows::*;
use self::schema::{local_album_mapping, local_albums, local_assets};
use super::folder::schema::folders;
use super::*;

mod dto;
mod rows;
pub(crate) mod schema;

pub use dto::*;

#[async_trait]
pub trait LocalStore: Send + Sync {
    /// Stage the files a walk of one local album found.
    ///
    /// `album_id` is the local album whose walk found them: a file the user
    /// picked by hand has none. A staged row is what that album's tile counts,
    /// so a directory whose files the album already holds stages its rows all
    /// the same.
    async fn save_local_assets(
        &self,
        album_id: Option<&str>,
        assets: &[LocalAsset],
    ) -> Result<(), CatalogError>;
    async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, CatalogError>;
    /// Every registered local album, each with the album it syncs into.
    async fn list_local_albums(&self) -> Result<Vec<LocalAlbumRecord>, CatalogError>;
    async fn get_local_album(&self, id: &str) -> Result<Option<LocalAlbumRecord>, CatalogError>;
    /// Register `uri` as a local album syncing into `album_id`.
    async fn add_local_album(
        &self,
        uri: &str,
        name: &str,
        album_id: &str,
    ) -> Result<(), CatalogError>;
    /// Forget a local album and its staging rows; the remote album stays.
    async fn remove_local_album(&self, id: &str) -> Result<(), CatalogError>;
    /// What this local album's tile shows: how many files it staged, and the
    /// newest four of them.
    async fn local_album_tile(&self, id: &str) -> Result<LocalAlbumTile, CatalogError>;
    /// Record that this directory was walked just now.
    ///
    /// A walk stages one row per file it found, and the tile is those rows, so
    /// the number moves while it runs. The row written here is the pass saying
    /// it is over: every file of the directory has been looked at.
    async fn mark_local_album_scanned(&self, id: &str) -> Result<(), CatalogError>;
}

/// Stage one page of a walking directory.
///
/// One statement per file, all in one transaction: a page is one commit, so the
/// tile sees the page whole, and a directory of ten thousand files costs fifty
/// commits instead of ten thousand.
pub(crate) async fn save_local_assets(
    cat: &SqliteCatalog,
    album_id: Option<&str>,
    assets: &[LocalAsset],
) -> Result<(), CatalogError> {
    if assets.is_empty() {
        return Ok(());
    }
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        for asset in assets {
            upsert_local_asset(conn, album_id, asset).await?;
        }
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

/// Insert or update one staged row.
///
/// A walk knows which album it is walking and writes that down; a file staged
/// by hand has no album and leaves the attribution a walk gave it alone, so
/// picking one photo out of a bound directory does not take it off that
/// directory's tile.
async fn upsert_local_asset(
    conn: &mut Conn,
    album_id: Option<&str>,
    asset: &LocalAsset,
) -> Result<(), CatalogError> {
    let insert = diesel::insert_into(local_assets::table).values((
        local_assets::id.eq(asset.id.0.clone()),
        local_assets::album_id.eq(album_id),
        local_assets::name.eq(asset.name.clone()),
        local_assets::mime.eq(asset.mime.clone()),
        local_assets::size.eq(asset.size),
        local_assets::date_taken.eq(asset.date_taken),
        local_assets::uri.eq(asset.uri.clone()),
        local_assets::width.eq(asset.width),
        local_assets::height.eq(asset.height),
        local_assets::duration_ms.eq(asset.duration_ms),
    ));
    let conflict = insert.on_conflict(local_assets::id).do_update();
    match album_id {
        Some(album_id) => conflict
            .set((
                local_assets::album_id.eq(album_id),
                local_assets::name.eq(excluded(local_assets::name)),
                local_assets::mime.eq(excluded(local_assets::mime)),
                local_assets::size.eq(excluded(local_assets::size)),
                local_assets::date_taken.eq(excluded(local_assets::date_taken)),
                local_assets::uri.eq(excluded(local_assets::uri)),
                local_assets::width.eq(excluded(local_assets::width)),
                local_assets::height.eq(excluded(local_assets::height)),
                local_assets::duration_ms.eq(excluded(local_assets::duration_ms)),
            ))
            .execute(conn)
            .await
            .map_err(db_err)?,
        None => conflict
            .set((
                local_assets::name.eq(excluded(local_assets::name)),
                local_assets::mime.eq(excluded(local_assets::mime)),
                local_assets::size.eq(excluded(local_assets::size)),
                local_assets::date_taken.eq(excluded(local_assets::date_taken)),
                local_assets::uri.eq(excluded(local_assets::uri)),
                local_assets::width.eq(excluded(local_assets::width)),
                local_assets::height.eq(excluded(local_assets::height)),
                local_assets::duration_ms.eq(excluded(local_assets::duration_ms)),
            ))
            .execute(conn)
            .await
            .map_err(db_err)?,
    };
    Ok(())
}

pub(crate) async fn get_local_asset(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<LocalAsset>, CatalogError> {
    let mut conn = cat.conn().await?;
    local_assets::table
        .filter(local_assets::id.eq(id))
        .select(LocalAssetRow::as_select())
        .first::<LocalAssetRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(LocalAsset::from))
        .map_err(db_err)
}

pub(crate) async fn list_local_albums(
    cat: &SqliteCatalog,
) -> Result<Vec<LocalAlbumRecord>, CatalogError> {
    let mut conn = cat.conn().await?;
    let rows = local_albums::table
        .inner_join(local_album_mapping::table.inner_join(folders::table))
        .select((
            local_albums::id,
            local_albums::uri,
            local_albums::name,
            local_album_mapping::album_id,
            folders::name,
        ))
        .load::<LocalAlbumRow>(&mut conn)
        .await
        .map_err(db_err)?;
    let mut albums: Vec<LocalAlbumRecord> = rows.into_iter().map(LocalAlbumRecord::from).collect();
    albums.sort_by_key(|album| album.name.to_lowercase());
    Ok(albums)
}

pub(crate) async fn get_local_album(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<LocalAlbumRecord>, CatalogError> {
    let mut conn = cat.conn().await?;
    local_albums::table
        .filter(local_albums::id.eq(id))
        .inner_join(local_album_mapping::table.inner_join(folders::table))
        .select((
            local_albums::id,
            local_albums::uri,
            local_albums::name,
            local_album_mapping::album_id,
            folders::name,
        ))
        .first::<LocalAlbumRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(LocalAlbumRecord::from))
        .map_err(db_err)
}

pub(crate) async fn add_local_album(
    cat: &SqliteCatalog,
    uri: &str,
    name: &str,
    album_id: &str,
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::insert_into(local_albums::table)
            .values((
                local_albums::id.eq(uri),
                local_albums::uri.eq(uri),
                local_albums::name.eq(name),
            ))
            .on_conflict(local_albums::id)
            .do_update()
            .set((
                local_albums::uri.eq(excluded(local_albums::uri)),
                local_albums::name.eq(excluded(local_albums::name)),
            ))
            .execute(&mut *conn)
            .await?;
        diesel::insert_into(local_album_mapping::table)
            .values((
                local_album_mapping::local_album_id.eq(uri),
                local_album_mapping::album_id.eq(album_id),
            ))
            .on_conflict(local_album_mapping::local_album_id)
            .do_update()
            .set(local_album_mapping::album_id.eq(excluded(local_album_mapping::album_id)))
            .execute(&mut *conn)
            .await?;
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

pub(crate) async fn remove_local_album(cat: &SqliteCatalog, id: &str) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::delete(
            local_album_mapping::table.filter(local_album_mapping::local_album_id.eq(id)),
        )
        .execute(&mut *conn)
        .await?;
        diesel::delete(local_assets::table.filter(local_assets::album_id.eq(id)))
            .execute(&mut *conn)
            .await?;
        diesel::delete(local_albums::table.filter(local_albums::id.eq(id)))
            .execute(&mut *conn)
            .await?;
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

pub(crate) async fn local_album_tile(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<LocalAlbumTile, CatalogError> {
    let mut conn = cat.conn().await?;
    let count = local_assets::table
        .filter(local_assets::album_id.eq(id))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(db_err)?
        .max(0) as u32;
    let cover_uris = local_assets::table
        .filter(local_assets::album_id.eq(id))
        .order((local_assets::date_taken.desc(), local_assets::id.desc()))
        .select(local_assets::uri)
        .limit(super::album::MOSAIC_COVERS)
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(LocalAlbumTile { count, cover_uris })
}

pub(crate) async fn mark_local_album_scanned(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    diesel::update(local_albums::table.filter(local_albums::id.eq(id)))
        .set(local_albums::scanned_at.eq(now_ms()))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

#[async_trait]
impl LocalStore for SqliteCatalog {
    async fn save_local_assets(
        &self,
        album_id: Option<&str>,
        assets: &[LocalAsset],
    ) -> Result<(), CatalogError> {
        save_local_assets(self, album_id, assets).await
    }

    async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, CatalogError> {
        get_local_asset(self, id).await
    }

    async fn list_local_albums(&self) -> Result<Vec<LocalAlbumRecord>, CatalogError> {
        list_local_albums(self).await
    }

    async fn get_local_album(&self, id: &str) -> Result<Option<LocalAlbumRecord>, CatalogError> {
        get_local_album(self, id).await
    }

    async fn add_local_album(
        &self,
        uri: &str,
        name: &str,
        album_id: &str,
    ) -> Result<(), CatalogError> {
        add_local_album(self, uri, name, album_id).await
    }

    async fn remove_local_album(&self, id: &str) -> Result<(), CatalogError> {
        remove_local_album(self, id).await
    }

    async fn local_album_tile(&self, id: &str) -> Result<LocalAlbumTile, CatalogError> {
        local_album_tile(self, id).await
    }

    async fn mark_local_album_scanned(&self, id: &str) -> Result<(), CatalogError> {
        mark_local_album_scanned(self, id).await
    }
}
