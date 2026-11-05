//! The local (on-device) media library: the contract and the diesel statements
//! behind it.
//!
//! Registration and mapping are written together: a local album exists only
//! because it syncs somewhere, so there is never a directory without a target.

use async_trait::async_trait;

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
    async fn save_local_asset(&self, asset: &LocalAsset) -> Result<(), AppError>;
    async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError>;
    /// Every registered local album, each with the album it syncs into.
    async fn list_local_albums(&self) -> Result<Vec<LocalAlbumRecord>, AppError>;
    async fn get_local_album(&self, id: &str) -> Result<Option<LocalAlbumRecord>, AppError>;
    /// Register `uri` as a local album syncing into `album_id`.
    async fn add_local_album(&self, uri: &str, name: &str, album_id: &str) -> Result<(), AppError>;
    /// Forget a local album and its staging rows; the remote album stays.
    async fn remove_local_album(&self, id: &str) -> Result<(), AppError>;
}

pub(crate) async fn save_local_asset(
    cat: &SqliteCatalog,
    asset: &LocalAsset,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::insert_into(local_assets::table)
        .values((
            local_assets::id.eq(asset.id.0.clone()),
            local_assets::album_id.eq(asset.album_id.clone()),
            local_assets::name.eq(asset.name.clone()),
            local_assets::mime.eq(asset.mime.clone()),
            local_assets::size.eq(asset.size),
            local_assets::date_taken.eq(asset.date_taken),
            local_assets::uri.eq(asset.uri.clone()),
            local_assets::width.eq(asset.width),
            local_assets::height.eq(asset.height),
            local_assets::duration_ms.eq(asset.duration_ms),
        ))
        .on_conflict(local_assets::id)
        .do_update()
        .set((
            local_assets::album_id.eq(excluded(local_assets::album_id)),
            local_assets::name.eq(excluded(local_assets::name)),
            local_assets::mime.eq(excluded(local_assets::mime)),
            local_assets::size.eq(excluded(local_assets::size)),
            local_assets::date_taken.eq(excluded(local_assets::date_taken)),
            local_assets::uri.eq(excluded(local_assets::uri)),
            local_assets::width.eq(excluded(local_assets::width)),
            local_assets::height.eq(excluded(local_assets::height)),
            local_assets::duration_ms.eq(excluded(local_assets::duration_ms)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn get_local_asset(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<LocalAsset>, AppError> {
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
) -> Result<Vec<LocalAlbumRecord>, AppError> {
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
) -> Result<Option<LocalAlbumRecord>, AppError> {
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
) -> Result<(), AppError> {
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

pub(crate) async fn remove_local_album(cat: &SqliteCatalog, id: &str) -> Result<(), AppError> {
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

#[async_trait]
impl LocalStore for SqliteCatalog {
    async fn save_local_asset(&self, asset: &LocalAsset) -> Result<(), AppError> {
        save_local_asset(self, asset).await
    }

    async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
        get_local_asset(self, id).await
    }

    async fn list_local_albums(&self) -> Result<Vec<LocalAlbumRecord>, AppError> {
        list_local_albums(self).await
    }

    async fn get_local_album(&self, id: &str) -> Result<Option<LocalAlbumRecord>, AppError> {
        get_local_album(self, id).await
    }

    async fn add_local_album(&self, uri: &str, name: &str, album_id: &str) -> Result<(), AppError> {
        add_local_album(self, uri, name, album_id).await
    }

    async fn remove_local_album(&self, id: &str) -> Result<(), AppError> {
        remove_local_album(self, id).await
    }
}
