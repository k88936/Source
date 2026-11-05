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

/// Which file a media element should load. The adapter turns it into an
/// address; nothing else in the app spells a scheme or a URL suffix.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum MediaSourceParams {
    /// Catalog thumbnail, a few hundred pixels wide.
    Thumb { asset_id: AssetId },
    /// Catalog original: the viewer's full-size still.
    Original { asset_id: AssetId },
    /// Catalog original played as media. Only the desktop adapter answers this
    /// one differently, because its media stack rejects custom schemes.
    Video { asset_id: AssetId },
    /// Thumbnail of a file that is not in the catalog.
    LocalThumb { uri: String },
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
    pub async fn get_timeline_page(
        &self,
        params: TimelinePageParams,
    ) -> Result<TimelinePage, AppError> {
        self.catalog
            .get_timeline_page(params.cursor.as_ref(), clamp_limit(params.limit))
            .await
    }

    pub async fn get_asset_detail(&self, asset_id: &str) -> Result<AssetDetail, AppError> {
        let record = self
            .catalog
            .get_asset(asset_id)
            .await?
            .ok_or_else(|| AppError::not_found("资产不存在"))?;
        let folder = self.catalog.get_folder(&record.summary.folder_id.0).await?;
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
            .get_asset(asset_id)
            .await?
            .map(|a| a.uri)
            .ok_or_else(|| AppError::not_found("资产不存在"))
    }

    pub async fn get_thumb_bytes(&self, asset_id: &str) -> Result<Vec<u8>, AppError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud.get_thumb_bytes(&uri).await
    }

    pub async fn get_media_bytes(&self, asset_id: &str) -> Result<Vec<u8>, AppError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud.get_media_bytes(&uri).await
    }

    pub async fn get_media_range(
        &self,
        asset_id: &str,
        range: Option<&str>,
    ) -> Result<MediaRange, AppError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud
            .get_media_range(&uri, bounded_range(range).as_deref())
            .await
    }

    /// Signed server URL of the original file, for the viewports that cannot
    /// play the `media` protocol themselves.
    pub async fn get_media_url(&self, asset_id: &str) -> Result<String, AppError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud.get_media_url(&uri).await
    }

    pub async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
        self.local.get_asset(id).await
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
            .get_asset(asset_id)
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
    pub async fn get_asset_thumb_hash(&self, asset_id: &str) -> Result<Option<String>, AppError> {
        Ok(self
            .catalog
            .get_asset(asset_id)
            .await?
            .and_then(|record| record.summary.thumb_hash))
    }

    pub async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), AppError> {
        self.catalog.set_asset_thumb_hash(asset_id, value).await
    }

    // ---------------------------------------------------------------------
    // Local library
    // ---------------------------------------------------------------------

    pub async fn get_media_permission_state(&self) -> Result<MediaPermissionState, AppError> {
        self.local.get_permission_state().await
    }

    pub async fn request_media_permission(&self) -> Result<MediaPermissionState, AppError> {
        self.local.request_permission().await
    }

    pub async fn get_local_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, AppError> {
        self.local.get_thumb(id, edge).await
    }
}

/// Bytes served by one range response. The webview answers are buffered whole,
/// so an open-ended `bytes=0-` would otherwise pull an entire video into memory.
const MEDIA_RANGE_CHUNK: u64 = 8 * 1024 * 1024;

/// Rewrites the open-ended `bytes=<start>-` the media players send into a
/// bounded window. Explicit ranges, suffix ranges and requests without a range
/// are forwarded untouched.
fn bounded_range(range: Option<&str>) -> Option<String> {
    let range = range?;
    let open_start = range
        .strip_prefix("bytes=")
        .and_then(|spec| spec.split_once('-'))
        .filter(|(start, end)| !start.is_empty() && end.is_empty())
        .and_then(|(start, _)| start.parse::<u64>().ok());
    match open_start {
        Some(start) => Some(format!("bytes={start}-{}", start + MEDIA_RANGE_CHUNK - 1)),
        None => Some(range.to_owned()),
    }
}
