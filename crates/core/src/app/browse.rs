//! Browsing the catalog, asset details and the local media library.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;
use crate::mapping::{ExifField, FAVORITE_KEY, exif_fields};
use crate::media::{LocalAsset, LocalMediaPort, MediaPermissionState};
use crate::remote::{MediaRange, RemoteStorage};
use crate::store::{
    AssetId, AssetStore, AssetSummary, FolderStore, MediaType, SearchQuery, TimelineCursor,
    TimelinePage, TimelineStore,
};

use super::clamp_limit;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelinePageParams {
    pub cursor: Option<TimelineCursor>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssetDetailParams {
    pub asset_id: AssetId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchParams {
    pub query: String,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub favorite: Option<bool>,
    pub media_type: Option<MediaType>,
    pub cursor: Option<TimelineCursor>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetFavoriteParams {
    pub asset_id: AssetId,
    pub value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssetDetail {
    pub asset: AssetSummary,
    pub album_name: Option<String>,
    pub album_path: Option<String>,
    pub exif: Vec<ExifField>,
    pub download_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportAssetsParams {
    pub asset_ids: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportResult {
    pub exported: u32,
    pub directory: String,
}

pub struct BrowseService<R, C, M> {
    cloud: R,
    catalog: C,
    local: M,
}

impl<R, C, M> BrowseService<R, C, M> {
    pub fn new(cloud: R, catalog: C, local: M) -> Self {
        Self {
            cloud,
            catalog,
            local,
        }
    }
}

impl<R, C, M> BrowseService<R, C, M>
where
    R: RemoteStorage,
    C: AssetStore + FolderStore + TimelineStore,
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

    async fn asset_uri(&self, asset_id: &str) -> Result<String, AppError> {
        self.catalog
            .asset(asset_id)
            .await?
            .map(|a| a.uri)
            .ok_or_else(|| AppError::not_found("资产不存在"))
    }

    pub async fn thumb_bytes(&self, asset_id: &str) -> Result<Vec<u8>, AppError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud.thumb_bytes(&uri).await
    }

    pub async fn media_bytes(&self, asset_id: &str) -> Result<Vec<u8>, AppError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud.media_bytes(&uri).await
    }

    pub async fn media_range(
        &self,
        asset_id: &str,
        range: Option<&str>,
    ) -> Result<MediaRange, AppError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud.media_range(&uri, range).await
    }

    pub fn local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
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

    pub fn local_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, AppError> {
        self.local.thumb(id, edge)
    }
}
