//! Browsing the catalog, asset details and the local media library.

mod dto;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::mapping::{FAVORITE_KEY, exif_fields};
use crate::media::{LocalAsset, LocalMediaPort, MediaError, MediaPermissionState};
use crate::remote::{MediaFetchError, MediaRange, MetadataError, RemoteStorage};
use crate::store::{
    AssetStore, AssetSummary, CatalogError, FolderStore, SearchQuery, TimelinePage, TimelineStore,
};

use super::clamp_limit;

pub use dto::*;

/// What browsing the catalog can fail with.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum BrowseError {
    /// The index could not answer.
    Catalog { cause: CatalogError },
    /// The row the UI asked for is no longer in the index.
    AssetMissing { asset: String },
    /// The server could not hand over this file.
    Media { asset: String, cause: MediaFetchError },
    /// The server refused the metadata patch.
    Metadata { asset: String, cause: MetadataError },
    /// The device media library failed.
    LocalMedia { cause: MediaError },
}

impl BrowseError {
    fn catalog(cause: CatalogError) -> Self {
        BrowseError::Catalog { cause }
    }
}

/// Exporting the selection to the download directory.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ExportError {
    /// The export folder could not be created.
    DirectoryUnwritable { cause: String },
    /// One file could not be written to disk.
    WriteFailed { path: String, cause: String },
    /// The asset itself could not be fetched.
    Asset { cause: BrowseError },
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
    ) -> Result<TimelinePage, BrowseError> {
        self.get_timeline_page_impl(params).await
    }

    pub async fn get_asset_detail(&self, asset_id: &str) -> Result<AssetDetail, BrowseError> {
        self.get_asset_detail_impl(asset_id).await
    }

    pub async fn get_thumb_bytes(&self, asset_id: &str) -> Result<Vec<u8>, BrowseError> {
        self.get_thumb_bytes_impl(asset_id).await
    }

    pub async fn get_media_bytes(&self, asset_id: &str) -> Result<Vec<u8>, BrowseError> {
        self.get_media_bytes_impl(asset_id).await
    }

    pub async fn get_media_range(
        &self,
        asset_id: &str,
        range: Option<&str>,
    ) -> Result<MediaRange, BrowseError> {
        self.get_media_range_impl(asset_id, range).await
    }

    /// Signed server URL of the original file, for the viewports that cannot
    /// play the `media` protocol themselves.
    pub async fn get_media_url(&self, asset_id: &str) -> Result<String, BrowseError> {
        self.get_media_url_impl(asset_id).await
    }

    pub async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, BrowseError> {
        self.get_local_asset_impl(id).await
    }

    pub async fn search(&self, params: SearchParams) -> Result<TimelinePage, BrowseError> {
        self.search_impl(params).await
    }

    pub async fn set_favorite(
        &self,
        asset_id: &str,
        value: bool,
    ) -> Result<AssetSummary, BrowseError> {
        self.set_favorite_impl(asset_id, value).await
    }

    /// Blur-up mosaic already stored for an asset, if any.
    pub async fn get_asset_thumb_hash(&self, asset_id: &str) -> Result<Option<String>, BrowseError> {
        self.get_asset_thumb_hash_impl(asset_id).await
    }

    pub async fn set_asset_thumb_hash(
        &self,
        asset_id: &str,
        value: &str,
    ) -> Result<(), BrowseError> {
        self.set_asset_thumb_hash_impl(asset_id, value).await
    }

    pub async fn get_media_permission_state(
        &self,
    ) -> Result<MediaPermissionState, BrowseError> {
        self.get_media_permission_state_impl().await
    }

    pub async fn request_media_permission(&self) -> Result<MediaPermissionState, BrowseError> {
        self.request_media_permission_impl().await
    }

    pub async fn get_local_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, BrowseError> {
        self.get_local_thumb_impl(id, edge).await
    }
}

// ---------------------------------------------------------------------
// Use case implementations
// ---------------------------------------------------------------------

impl<R, C, M> BrowseService<R, C, M>
where
    R: RemoteStorage,
    C: AssetStore + FolderStore + TimelineStore,
    M: LocalMediaPort,
{
    async fn get_timeline_page_impl(
        &self,
        params: TimelinePageParams,
    ) -> Result<TimelinePage, BrowseError> {
        self.catalog
            .get_timeline_page(params.cursor.as_ref(), clamp_limit(params.limit))
            .await
            .map_err(BrowseError::catalog)
    }

    async fn get_asset_detail_impl(&self, asset_id: &str) -> Result<AssetDetail, BrowseError> {
        let record = self
            .catalog
            .get_asset(asset_id)
            .await
            .map_err(BrowseError::catalog)?
            .ok_or_else(|| BrowseError::AssetMissing {
                asset: asset_id.to_string(),
            })?;
        let folder = self
            .catalog
            .get_folder(&record.summary.folder_id.0)
            .await
            .map_err(BrowseError::catalog)?;
        Ok(AssetDetail {
            download_name: record.summary.name.clone(),
            exif: exif_fields(record.exif_json.as_deref()),
            album_name: folder.as_ref().map(|f| f.name.clone()),
            album_path: folder.as_ref().map(|f| f.path.clone()),
            asset: record.summary,
        })
    }

    async fn get_thumb_bytes_impl(&self, asset_id: &str) -> Result<Vec<u8>, BrowseError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud
            .get_thumb_bytes(&uri)
            .await
            .map_err(|cause| BrowseError::Media {
                asset: asset_id.to_string(),
                cause,
            })
    }

    async fn get_media_bytes_impl(&self, asset_id: &str) -> Result<Vec<u8>, BrowseError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud
            .get_media_bytes(&uri)
            .await
            .map_err(|cause| BrowseError::Media {
                asset: asset_id.to_string(),
                cause,
            })
    }

    async fn get_media_range_impl(
        &self,
        asset_id: &str,
        range: Option<&str>,
    ) -> Result<MediaRange, BrowseError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud
            .get_media_range(&uri, bounded_range(range).as_deref())
            .await
            .map_err(|cause| BrowseError::Media {
                asset: asset_id.to_string(),
                cause,
            })
    }

    async fn get_media_url_impl(&self, asset_id: &str) -> Result<String, BrowseError> {
        let uri = self.asset_uri(asset_id).await?;
        self.cloud
            .get_media_url(&uri)
            .await
            .map_err(|cause| BrowseError::Media {
                asset: asset_id.to_string(),
                cause,
            })
    }

    async fn get_local_asset_impl(&self, id: &str) -> Result<Option<LocalAsset>, BrowseError> {
        self.local
            .get_asset(id)
            .await
            .map_err(|cause| BrowseError::LocalMedia { cause })
    }

    async fn search_impl(&self, params: SearchParams) -> Result<TimelinePage, BrowseError> {
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
            .map_err(BrowseError::catalog)
    }

    async fn set_favorite_impl(
        &self,
        asset_id: &str,
        value: bool,
    ) -> Result<AssetSummary, BrowseError> {
        let record = self
            .catalog
            .get_asset(asset_id)
            .await
            .map_err(BrowseError::catalog)?
            .ok_or_else(|| BrowseError::AssetMissing {
                asset: asset_id.to_string(),
            })?;
        self.catalog
            .set_favorite(asset_id, value)
            .await
            .map_err(BrowseError::catalog)?;
        let patch = (
            FAVORITE_KEY.to_string(),
            Some(if value { "1" } else { "0" }.to_string()),
        );
        if let Err(cause) = self.cloud.patch_metadata(&record.uri, &[patch]).await {
            self.catalog
                .set_favorite(asset_id, record.summary.favorite)
                .await
                .map_err(BrowseError::catalog)?;
            return Err(BrowseError::Metadata {
                asset: asset_id.to_string(),
                cause,
            });
        }
        Ok(AssetSummary {
            favorite: value,
            ..record.summary
        })
    }

    async fn get_asset_thumb_hash_impl(&self, asset_id: &str) -> Result<Option<String>, BrowseError> {
        Ok(self
            .catalog
            .get_asset(asset_id)
            .await
            .map_err(BrowseError::catalog)?
            .and_then(|record| record.summary.thumb_hash))
    }

    async fn set_asset_thumb_hash_impl(
        &self,
        asset_id: &str,
        value: &str,
    ) -> Result<(), BrowseError> {
        self.catalog
            .set_asset_thumb_hash(asset_id, value)
            .await
            .map_err(BrowseError::catalog)
    }

    async fn get_media_permission_state_impl(&self) -> Result<MediaPermissionState, BrowseError> {
        self.local
            .get_permission_state()
            .await
            .map_err(|cause| BrowseError::LocalMedia { cause })
    }

    async fn request_media_permission_impl(&self) -> Result<MediaPermissionState, BrowseError> {
        self.local
            .request_permission()
            .await
            .map_err(|cause| BrowseError::LocalMedia { cause })
    }

    async fn get_local_thumb_impl(&self, id: &str, edge: u32) -> Result<Vec<u8>, BrowseError> {
        self.local
            .get_thumb(id, edge)
            .await
            .map_err(|cause| BrowseError::LocalMedia { cause })
    }

    async fn asset_uri(&self, asset_id: &str) -> Result<String, BrowseError> {
        self.catalog
            .get_asset(asset_id)
            .await
            .map_err(BrowseError::catalog)?
            .map(|a| a.uri)
            .ok_or_else(|| BrowseError::AssetMissing {
                asset: asset_id.to_string(),
            })
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
