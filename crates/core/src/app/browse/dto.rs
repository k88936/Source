//! Parameters the browse use cases accept, and the payloads they return.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::mapping::ExifField;
use crate::store::{AssetId, AssetSummary, FolderId, MediaType, TimelineCursor};

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
    /// Restrict the search to one album; `None` searches the whole library.
    pub folder_id: Option<FolderId>,
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
