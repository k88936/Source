//! Albums: the remote folders a user added, with derived counts and covers.
//! The contract and the diesel statements behind it.

use std::collections::HashMap;

use async_trait::async_trait;

use super::asset::schema::assets;
use super::folder::rows::FolderRow;
use super::folder::schema::{folders, sync_state};
use super::local::schema::{local_album_mapping, local_albums};
use super::*;

mod dto;

pub use dto::*;

#[async_trait]
pub trait AlbumStore: Send + Sync {
    /// Store the folder a user added, or refresh its name and URI.
    async fn save_album(&self, album: &FolderRecord) -> Result<(), CatalogError>;
    /// Forget an album, its assets and every local album syncing into it.
    async fn delete_album(&self, id: &str) -> Result<(), CatalogError>;
    async fn list_albums(&self) -> Result<Vec<AlbumMetadata>, CatalogError>;
    async fn get_album(&self, id: &str) -> Result<Option<AlbumMetadata>, CatalogError>;
    async fn set_album_cover(
        &self,
        album_id: &str,
        cover: Option<&str>,
    ) -> Result<(), CatalogError>;
    async fn set_album_description(
        &self,
        album_id: &str,
        description: Option<&str>,
    ) -> Result<(), CatalogError>;
}

/// How many covers the album mosaic shows.
const MOSAIC_COVERS: i64 = 4;

/// Add or refresh the album a user picked, keeping cover and description.
pub(crate) async fn save_album(
    cat: &SqliteCatalog,
    album: &FolderRecord,
) -> Result<(), CatalogError> {
    upsert_folder(cat, album).await
}

/// Forget an album: its assets, its covers, its local sync targets and the
/// local albums that were syncing into it.
///
/// The remote folder is never touched; only the catalog forgets it, so the
/// same folder can be added again later.
pub(crate) async fn delete_album(cat: &SqliteCatalog, id: &str) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    let locals = local_album_mapping::table
        .filter(local_album_mapping::album_id.eq(id))
        .select(local_album_mapping::local_album_id)
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)?;

    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::delete(local_album_mapping::table.filter(local_album_mapping::album_id.eq(id)))
            .execute(&mut *conn)
            .await?;
        if !locals.is_empty() {
            diesel::delete(local_albums::table.filter(local_albums::id.eq_any(&locals)))
                .execute(&mut *conn)
                .await?;
        }
        diesel::delete(sync_state::table.filter(sync_state::folder_id.eq(id)))
            .execute(&mut *conn)
            .await?;
        diesel::delete(folders::table.filter(folders::id.eq(id)))
            .execute(&mut *conn)
            .await?;
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

/// The last listing outcome of every album in `ids`, keyed by album id.
async fn sync_states(
    conn: &mut Conn,
    ids: &[String],
) -> Result<HashMap<String, AlbumSync>, CatalogError> {
    let rows = sync_state::table
        .filter(sync_state::folder_id.eq_any(ids))
        .select((
            sync_state::folder_id,
            sync_state::status,
            sync_state::last_listed_at,
        ))
        .load::<(String, String, Option<i64>)>(conn)
        .await
        .map_err(db_err)?;
    Ok(rows
        .into_iter()
        .map(|(id, status, listed_at)| {
            (
                id,
                AlbumSync {
                    failed: status != SYNC_DONE,
                    listed_at,
                },
            )
        })
        .collect())
}

pub(crate) async fn list_albums(cat: &SqliteCatalog) -> Result<Vec<AlbumMetadata>, CatalogError> {
    let mut conn = cat.conn().await?;
    let mut albums = folders::table
        .select(FolderRow::as_select())
        .load::<FolderRow>(&mut conn)
        .await
        .map_err(db_err)?;
    // `ORDER BY name COLLATE NOCASE` has no DSL equivalent in diesel, and the
    // set is small enough to sort here.
    albums.sort_by_key(|album| album.name.to_lowercase());

    let ids: Vec<String> = albums.iter().map(|album| album.id.clone()).collect();
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut sync = sync_states(&mut conn, &ids).await?;

    let counts = assets::table
        .filter(assets::folder_id.eq_any(&ids))
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .group_by(assets::folder_id)
        .select((assets::folder_id, diesel::dsl::count_star()))
        .load::<(String, i64)>(&mut conn)
        .await
        .map_err(db_err)?;

    // The album tile shows a 2x2 mosaic, so each album needs up to four
    // covers: the explicit one first, then the newest media. One indexed
    // `LIMIT 4` query per album stays cheap and keeps the statement plain DSL
    // (a correlated `ROW_NUMBER()` window would be the only alternative).
    let mut covers: HashMap<String, Vec<String>> = HashMap::new();
    for album in &albums {
        let asset_count = counts
            .iter()
            .find(|(id, _)| *id == album.id)
            .map(|(_, count)| *count)
            .unwrap_or(0);
        if asset_count <= 0 {
            continue;
        }
        let mut ids = assets::table
            .filter(assets::folder_id.eq(&album.id))
            .filter(assets::media_type.eq_any(MEDIA_TYPES))
            .order((assets::taken_at.desc(), assets::id.desc()))
            .select(assets::id)
            .limit(MOSAIC_COVERS)
            .load::<String>(&mut conn)
            .await
            .map_err(db_err)?;
        if let Some(cover) = &album.cover_asset_id {
            ids.retain(|id| id != cover);
            ids.insert(0, cover.clone());
            ids.truncate(MOSAIC_COVERS as usize);
        }
        covers.insert(album.id.clone(), ids);
    }

    Ok(albums
        .into_iter()
        .map(|album| {
            let asset_count = counts
                .iter()
                .find(|(id, _)| *id == album.id)
                .map(|(_, count)| (*count).max(0) as u32)
                .unwrap_or(0);
            let cover_ids = covers
                .remove(&album.id)
                .unwrap_or_default()
                .into_iter()
                .map(AssetId)
                .collect();
            let sync = sync.remove(&album.id);
            AlbumMetadata {
                id: AlbumId(album.id),
                name: album.name,
                uri: album.uri,
                path: display_path(&album.path),
                asset_count,
                cover_asset_ids: cover_ids,
                description: album.description,
                sync,
                updated_at: album.updated_at,
            }
        })
        .collect())
}

/// One album with its covers. Queries the row directly instead of scanning
/// [`album_list`], so opening an album costs three indexed statements and not
/// one per album in the library.
pub(crate) async fn get_album(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<AlbumMetadata>, CatalogError> {
    let mut conn = cat.conn().await?;
    let Some(folder) = folders::table
        .filter(folders::id.eq(id))
        .select(FolderRow::as_select())
        .first::<FolderRow>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?
    else {
        return Ok(None);
    };

    let asset_count = assets::table
        .filter(assets::folder_id.eq(id))
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(db_err)?
        .max(0) as u32;

    let mut covers = assets::table
        .filter(assets::folder_id.eq(id))
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .order((assets::taken_at.desc(), assets::id.desc()))
        .select(assets::id)
        .limit(MOSAIC_COVERS)
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)?;
    if let Some(cover) = &folder.cover_asset_id {
        covers.retain(|id| id != cover);
        covers.insert(0, cover.clone());
        covers.truncate(MOSAIC_COVERS as usize);
    }

    let sync = sync_states(&mut conn, &[id.to_string()]).await?.remove(id);

    Ok(Some(AlbumMetadata {
        id: AlbumId(folder.id),
        name: folder.name,
        uri: folder.uri,
        path: display_path(&folder.path),
        asset_count,
        cover_asset_ids: covers.into_iter().map(AssetId).collect(),
        description: folder.description,
        sync,
        updated_at: folder.updated_at,
    }))
}

pub(crate) async fn set_album_cover(
    cat: &SqliteCatalog,
    album_id: &str,
    cover: Option<&str>,
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    diesel::update(folders::table.filter(folders::id.eq(album_id)))
        .set(folders::cover_asset_id.eq(cover))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn set_album_description(
    cat: &SqliteCatalog,
    album_id: &str,
    description: Option<&str>,
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    diesel::update(folders::table.filter(folders::id.eq(album_id)))
        .set(folders::description.eq(description))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

#[async_trait]
impl AlbumStore for SqliteCatalog {
    async fn save_album(&self, album: &FolderRecord) -> Result<(), CatalogError> {
        save_album(self, album).await
    }

    async fn delete_album(&self, id: &str) -> Result<(), CatalogError> {
        delete_album(self, id).await
    }

    async fn list_albums(&self) -> Result<Vec<AlbumMetadata>, CatalogError> {
        list_albums(self).await
    }

    async fn get_album(&self, id: &str) -> Result<Option<AlbumMetadata>, CatalogError> {
        get_album(self, id).await
    }

    async fn set_album_cover(
        &self,
        album_id: &str,
        cover: Option<&str>,
    ) -> Result<(), CatalogError> {
        set_album_cover(self, album_id, cover).await
    }

    async fn set_album_description(
        &self,
        album_id: &str,
        description: Option<&str>,
    ) -> Result<(), CatalogError> {
        set_album_description(self, album_id, description).await
    }
}
