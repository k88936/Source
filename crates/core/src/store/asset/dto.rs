//! Asset payloads: the indexed summary the UI reads, the full record the
//! indexer writes, and the enum columns the rows encode as strings.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::super::folder::FolderId;
use crate::store::id_type;

id_type!(AssetId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TakenAtSource {
    Exif,
    ClientExif,
    UpdatedAt,
    Filename,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssetSummary {
    pub id: AssetId,
    pub folder_id: FolderId,
    pub name: String,
    pub media_type: MediaType,
    pub mime: Option<String>,
    pub size: i64,
    pub taken_at: i64,
    pub taken_at_source: TakenAtSource,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub favorite: bool,
    /// Base64 of a tiny downscaled preview, used as a blur-up placeholder.
    /// Filled lazily the first time the thumbnail is requested.
    pub thumb_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetRecord {
    pub summary: AssetSummary,
    /// Cloudreve file URI, needed by the thumbnail/media proxies.
    pub uri: String,
    /// Raw `exif:*` metadata as JSON, for the detail sheet.
    pub exif_json: Option<String>,
    pub taken_at_source: TakenAtSource,
    pub remote_updated_at: i64,
}
