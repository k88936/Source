//! Events the use cases emit to the UI, and the parameters they accept.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::{AlbumSummary, AssetSummary};

/// What `app_info` reports about the running build.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
}

/// What a `catalog://changed` event invalidates.
///
/// `Folders` is the coarse signal emitted after an index pass finished; the
/// other two carry the affected rows so the UI can patch instead of reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CatalogChangeKind {
    Assets,
    Albums,
    Folders,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CatalogChangedEvent {
    pub kind: CatalogChangeKind,
    #[serde(default)]
    pub assets: Vec<AssetSummary>,
    #[serde(default)]
    pub albums: Vec<AlbumSummary>,
}
