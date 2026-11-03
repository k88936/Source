//! Albums: folders that contain media, with derived counts and covers. The
//! contract and the diesel statements behind it.

use std::collections::HashMap;

use async_trait::async_trait;

use super::asset::schema::assets;
use super::folder::rows::FolderRow;
use super::folder::schema::folders;
use super::query::*;
use super::*;

mod dto;

pub use dto::*;

#[async_trait]
pub trait AlbumStore: Send + Sync {
    async fn mark_album(&self, folder_id: &str, is_album: bool) -> Result<(), AppError>;
    async fn rebuild_albums(&self) -> Result<(), AppError>;
    async fn album_list(&self) -> Result<Vec<AlbumSummary>, AppError>;
    async fn album(&self, id: &str) -> Result<Option<AlbumSummary>, AppError>;
    async fn album_page(
        &self,
        album_id: &str,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError>;
    async fn set_album_cover(&self, album_id: &str, cover: Option<&str>) -> Result<(), AppError>;
    async fn set_album_description(
        &self,
        album_id: &str,
        description: Option<&str>,
    ) -> Result<(), AppError>;
}

/// How many covers the album mosaic shows.
const MOSAIC_COVERS: i64 = 4;

pub(crate) async fn album_list(cat: &SqliteCatalog) -> Result<Vec<AlbumSummary>, AppError> {
    let mut conn = cat.conn().await?;
    let mut albums = folders::table
        .filter(folders::is_album.eq(true))
        .select(FolderRow::as_select())
        .load::<FolderRow>(&mut conn)
        .await
        .map_err(db_err)?;
    // `ORDER BY path COLLATE NOCASE` has no DSL equivalent in diesel, and
    // the set is small enough to sort here.
    albums.sort_by_key(|album| album.path.to_lowercase());

    let ids: Vec<String> = albums.iter().map(|album| album.id.clone()).collect();
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let counts = assets::table
        .filter(assets::folder_id.eq_any(&ids))
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .group_by(assets::folder_id)
        .select((assets::folder_id, diesel::dsl::count_star()))
        .load::<(String, i64)>(&mut conn)
        .await
        .map_err(db_err)?;

    // The album tile shows a 2×2 mosaic, so each album needs up to four
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
            AlbumSummary {
                id: AlbumId(album.id),
                name: album.name,
                uri: album.uri,
                path: album.path,
                asset_count,
                cover_asset_ids: cover_ids,
                description: album.description,
                updated_at: album.updated_at,
            }
        })
        .collect())
}

/// One album with its covers. Queries the row directly instead of scanning
/// [`album_list`], so opening an album costs three indexed statements and not
/// one per album in the library.
pub(crate) async fn album(cat: &SqliteCatalog, id: &str) -> Result<Option<AlbumSummary>, AppError> {
    let mut conn = cat.conn().await?;
    let Some(folder) = folders::table
        .filter(folders::id.eq(id))
        .filter(folders::is_album.eq(true))
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

    Ok(Some(AlbumSummary {
        id: AlbumId(folder.id),
        name: folder.name,
        uri: folder.uri,
        path: folder.path,
        asset_count,
        cover_asset_ids: covers.into_iter().map(AssetId).collect(),
        description: folder.description,
        updated_at: folder.updated_at,
    }))
}

pub(crate) async fn album_page(
    cat: &SqliteCatalog,
    album_id: &str,
    cursor: Option<&TimelineCursor>,
    limit: u32,
) -> Result<TimelinePage, AppError> {
    let mut conn = cat.conn().await?;
    timeline(&mut conn, Some(album_id), cursor, limit).await
}

pub(crate) async fn set_album_cover(
    cat: &SqliteCatalog,
    album_id: &str,
    cover: Option<&str>,
) -> Result<(), AppError> {
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
) -> Result<(), AppError> {
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
    async fn mark_album(&self, folder_id: &str, is_album: bool) -> Result<(), AppError> {
        super::asset::mark_album(self, folder_id, is_album).await
    }

    async fn rebuild_albums(&self) -> Result<(), AppError> {
        super::asset::rebuild_albums(self).await
    }

    async fn album_list(&self) -> Result<Vec<AlbumSummary>, AppError> {
        album_list(self).await
    }

    async fn album(&self, id: &str) -> Result<Option<AlbumSummary>, AppError> {
        album(self, id).await
    }

    async fn album_page(
        &self,
        album_id: &str,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError> {
        album_page(self, album_id, cursor, limit).await
    }

    async fn set_album_cover(&self, album_id: &str, cover: Option<&str>) -> Result<(), AppError> {
        set_album_cover(self, album_id, cover).await
    }

    async fn set_album_description(
        &self,
        album_id: &str,
        description: Option<&str>,
    ) -> Result<(), AppError> {
        set_album_description(self, album_id, description).await
    }
}
