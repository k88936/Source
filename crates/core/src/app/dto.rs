//! Events no single service owns, and what the running build reports about
//! itself.

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
/// `Albums` is the coarse signal for a listing pass and carries the refreshed
/// rows; `Assets` carries the rows an upload or a favorite wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CatalogChangeKind {
    Assets,
    Albums,
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
