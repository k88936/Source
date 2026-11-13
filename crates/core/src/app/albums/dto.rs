//! Parameters the album use cases accept, and the metadata they return.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::{AlbumId, AssetId};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddAlbumParams {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlbumParams {
    pub album_id: AlbumId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetAlbumCoverParams {
    pub album_id: AlbumId,
    pub asset_id: AssetId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetAlbumDescriptionParams {
    pub album_id: AlbumId,
    pub description: String,
}

/// Move a batch of assets into an album (a folder move on Cloudreve).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveAssetsParams {
    pub asset_ids: Vec<AssetId>,
    pub album_id: AlbumId,
}

/// A registered local album together with where it syncs to.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAlbumMetadata {
    pub id: String,
    pub uri: String,
    pub name: String,
    pub album_id: AlbumId,
    pub album_name: String,
    /// Direct media files, counted on the device every time it is asked.
    pub asset_count: u32,
    /// Up to four newest local files, for the mosaic tile.
    pub cover_uris: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddLocalAlbumParams {
    /// The directory as the media adapter names it.
    pub uri: String,
    /// What the picker called it; kept so a bucket rename does not blank the row.
    pub name: String,
    pub album_id: AlbumId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAlbumParams {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalBrowseParams {
    /// The directory to list; `None` starts where the platform browses from.
    pub parent: Option<String>,
}
