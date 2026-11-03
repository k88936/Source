//! Parameter structs for catalog queries and mutations.

use super::*;

// ---------------------------------------------------------------------------
// Catalog / browsing params
// ---------------------------------------------------------------------------

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
pub struct AlbumPageParams {
    pub album_id: AlbumId,
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
pub struct SetAlbumCoverParams {
    pub album_id: AlbumId,
    pub asset_id: AssetId,
}

/// Move a batch of assets into an album (a folder move on Cloudreve).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveAssetsParams {
    pub asset_ids: Vec<AssetId>,
    pub album_id: AlbumId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetAlbumDescriptionParams {
    pub album_id: AlbumId,
    pub description: String,
}
