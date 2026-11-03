//! Browsing the catalog, asset details and the local media library.

use super::*;

impl<C, R, M> AppCore<C, R, M>
where
    C: AlbumStore + AssetStore + FolderStore + TimelineStore,
    R: CloudrevePort,
    M: LocalMediaPort,
{
    pub async fn timeline_page(
        &self,
        params: TimelinePageParams,
    ) -> Result<TimelinePage, AppError> {
        self.catalog
            .timeline_page(params.cursor.as_ref(), clamp_limit(params.limit))
            .await
    }

    pub async fn album_list(&self) -> Result<Vec<AlbumSummary>, AppError> {
        self.catalog.album_list().await
    }

    pub async fn album_page(&self, params: AlbumPageParams) -> Result<TimelinePage, AppError> {
        self.catalog
            .album_page(
                &params.album_id.0,
                params.cursor.as_ref(),
                clamp_limit(params.limit),
            )
            .await
    }

    pub async fn album(&self, album_id: &str) -> Result<Option<AlbumSummary>, AppError> {
        self.catalog.album(album_id).await
    }

    pub async fn asset_detail(&self, asset_id: &str) -> Result<AssetDetail, AppError> {
        let record = self
            .catalog
            .asset(asset_id)
            .await?
            .ok_or_else(|| AppError::not_found("资产不存在"))?;
        let folder = self.catalog.folder(&record.summary.folder_id.0).await?;
        Ok(AssetDetail {
            download_name: record.summary.name.clone(),
            exif: exif_fields(record.exif_json.as_deref()),
            album_name: folder.as_ref().map(|f| f.name.clone()),
            album_path: folder.as_ref().map(|f| f.path.clone()),
            asset: record.summary,
        })
    }

    pub async fn asset_uri(&self, asset_id: &str) -> Result<Option<String>, AppError> {
        Ok(self.catalog.asset(asset_id).await?.map(|a| a.uri))
    }

    pub async fn thumb_url(&self, asset_id: &str) -> Result<String, AppError> {
        let uri = self
            .asset_uri(asset_id)
            .await?
            .ok_or_else(|| AppError::not_found("资产不存在"))?;
        self.cloud.thumb_url(&uri).await
    }

    pub async fn media_url(&self, asset_id: &str) -> Result<String, AppError> {
        let uri = self
            .asset_uri(asset_id)
            .await?
            .ok_or_else(|| AppError::not_found("资产不存在"))?;
        self.cloud.media_url(&uri).await
    }

    pub fn local_asset(&self, id: &str) -> Result<Option<crate::dto::LocalAsset>, AppError> {
        self.local.asset(id)
    }

    pub async fn search(&self, params: SearchParams) -> Result<TimelinePage, AppError> {
        self.catalog
            .search(&SearchQuery {
                query: params.query,
                from: params.from,
                to: params.to,
                favorite: params.favorite,
                media_type: params.media_type,
                cursor: params.cursor,
                limit: clamp_limit(params.limit),
            })
            .await
    }

    pub async fn set_favorite(
        &self,
        asset_id: &str,
        value: bool,
    ) -> Result<AssetSummary, AppError> {
        let record = self
            .catalog
            .asset(asset_id)
            .await?
            .ok_or_else(|| AppError::not_found("资产不存在"))?;
        self.catalog.set_favorite(asset_id, value).await?;
        let patch = (
            FAVORITE_KEY.to_string(),
            Some(if value { "1" } else { "0" }.to_string()),
        );
        if let Err(err) = self.cloud.patch_metadata(&record.uri, &[patch]).await {
            self.catalog
                .set_favorite(asset_id, record.summary.favorite)
                .await?;
            return Err(err);
        }
        Ok(AssetSummary {
            favorite: value,
            ..record.summary
        })
    }

    pub async fn set_album_cover(
        &self,
        album_id: &str,
        asset_id: &str,
    ) -> Result<AlbumSummary, AppError> {
        let folder = self
            .catalog
            .folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let patch = (COVER_KEY.to_string(), Some(asset_id.to_string()));
        self.cloud.patch_metadata(&folder.uri, &[patch]).await?;
        self.catalog
            .set_album_cover(album_id, Some(asset_id))
            .await?;
        self.catalog
            .album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    pub async fn set_album_description(
        &self,
        album_id: &str,
        description: &str,
    ) -> Result<AlbumSummary, AppError> {
        let folder = self
            .catalog
            .folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let value = if description.trim().is_empty() {
            None
        } else {
            Some(description.to_string())
        };
        self.cloud
            .patch_metadata(&folder.uri, &[(DESCRIPTION_KEY.to_string(), value.clone())])
            .await?;
        self.catalog
            .set_album_description(album_id, value.as_deref())
            .await?;
        self.catalog
            .album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    /// Move assets into an album.
    ///
    /// In Cloudreve an album is a folder, so this is a real move: the files
    /// leave their current folder. The catalog rows are re-pointed only after
    /// the remote move succeeded.
    pub async fn move_assets_to_album(
        &self,
        asset_ids: &[String],
        album_id: &str,
    ) -> Result<Vec<AssetSummary>, AppError> {
        if asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let folder = self
            .catalog
            .folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let mut records = Vec::with_capacity(asset_ids.len());
        for id in asset_ids {
            let record = self
                .catalog
                .asset(id)
                .await?
                .ok_or_else(|| AppError::not_found("资产不存在"))?;
            records.push(record);
        }
        let uris: Vec<String> = records.iter().map(|r| r.uri.clone()).collect();
        let new_uris = self.cloud.move_assets(&uris, &folder.uri).await?;
        if new_uris.len() != records.len() {
            return Err(AppError::internal("移动结果与请求不一致"));
        }

        let album_id = crate::dto::FolderId::new(folder.id);
        let mut moved = Vec::with_capacity(records.len());
        for (record, new_uri) in records.into_iter().zip(new_uris) {
            let asset_id = record.summary.id.0.clone();
            self.catalog
                .move_asset(&asset_id, &album_id.0, &new_uri)
                .await?;
            moved.push(AssetSummary {
                folder_id: album_id.clone(),
                ..record.summary
            });
        }
        self.catalog.rebuild_albums().await?;
        Ok(moved)
    }

    /// Blur-up mosaic already stored for an asset, if any.
    pub async fn asset_thumb_hash(&self, asset_id: &str) -> Result<Option<String>, AppError> {
        Ok(self
            .catalog
            .asset(asset_id)
            .await?
            .and_then(|record| record.summary.thumb_hash))
    }

    pub async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), AppError> {
        self.catalog.set_asset_thumb_hash(asset_id, value).await
    }

    // ---------------------------------------------------------------------
    // Local library
    // ---------------------------------------------------------------------

    pub fn media_permission_state(&self) -> MediaPermissionState {
        self.local.permission_state()
    }

    pub fn request_media_permission(&self) -> MediaPermissionState {
        self.local.request_permission()
    }

    pub fn local_album_list(&self) -> Result<Vec<LocalAlbum>, AppError> {
        self.local.albums()
    }

    pub fn local_asset_page(
        &self,
        params: LocalAssetPageParams,
    ) -> Result<crate::dto::LocalAssetPage, AppError> {
        self.local.assets(
            params.album_id.as_deref(),
            params.cursor.as_deref(),
            clamp_limit(params.limit),
        )
    }

    pub fn local_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, AppError> {
        self.local.thumb(id, edge)
    }
}
