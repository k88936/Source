//! Cache maintenance and change events.

use super::*;

// ---------------------------------------------------------------------------
// Maintenance
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
