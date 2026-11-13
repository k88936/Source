//! Album payloads: an album row as the UI sees it, with counts and covers.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::super::asset::AssetId;
use crate::store::id_type;

id_type!(AlbumId);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlbumMetadata {
    pub id: AlbumId,
    pub name: String,
    pub uri: String,
    pub path: String,
    pub asset_count: u32,
    /// Up to four newest covers, explicit cover first, for the mosaic tile.
    pub cover_asset_ids: Vec<AssetId>,
    pub description: Option<String>,
    pub updated_at: i64,
    /// Outcome of the last listing; absent while the album was never listed.
    pub sync: Option<AlbumSync>,
}

/// How the last listing of an album went.
///
/// A folder can disappear or be renamed on the server without the album row
/// going anywhere, so the row keeps its assets and reports the failure instead:
/// the album stays visible and the user can point it at another folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlbumSync {
    pub failed: bool,
    pub listed_at: Option<i64>,
}
