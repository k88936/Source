//! What the local library reports and accepts: permission state, the
//! directories it can offer, assets and the thumbnail cache counters.

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

/// One directory the user can add as a local album.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalEntry {
    /// What the adapter calls this directory; the registered album keeps it.
    pub uri: String,
    pub name: String,
    /// Where the directory sits, for platforms whose names collide
    /// (`DCIM/Camera` on Android). The desktop name is the last component.
    pub path: Option<String>,
}

/// One step of the local directory browser.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalBrowsePage {
    /// The directory being listed; `None` means the platform has no notion of
    /// a current directory (it only offers a list to pick from).
    pub current: Option<LocalEntry>,
    /// Where "上一级" goes; `None` at the top.
    pub parent: Option<LocalEntry>,
    pub entries: Vec<LocalEntry>,
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
    pub album_uri: String,
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
