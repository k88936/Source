//! Album payloads: an album row as the UI sees it, with counts and covers.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::super::asset::AssetId;
use crate::store::id_type;

id_type!(AlbumId);

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlbumSummary {
    pub id: AlbumId,
    pub name: String,
    pub uri: String,
    pub path: String,
    pub asset_count: u32,
    /// Up to four newest covers, explicit cover first, for the mosaic tile.
    pub cover_asset_ids: Vec<AssetId>,
    pub description: Option<String>,
    pub updated_at: i64,
}
