//! What the local library reports and accepts: permission state, albums,
//! assets and the thumbnail cache counters.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PermissionGrant {
    Full,
    Partial,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MediaPermissionState {
    pub granted: PermissionGrant,
    pub can_request: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CacheStats {
    pub thumb_bytes: i64,
    pub thumb_files: u32,
    pub media_bytes: i64,
    pub media_files: u32,
    pub budget_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAlbum {
    pub id: String,
    pub name: String,
    pub asset_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAsset {
    pub id: String,
    pub album_id: Option<String>,
    pub name: String,
    pub mime: Option<String>,
    pub size: i64,
    pub date_taken: i64,
    pub uri: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAssetPageParams {
    pub album_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAssetPage {
    pub items: Vec<LocalAsset>,
    pub next: Option<String>,
}
