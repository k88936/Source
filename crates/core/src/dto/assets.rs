//! Catalog entities: assets, albums and timeline pages.

use super::*;

// ---------------------------------------------------------------------------
// Assets & albums
// ---------------------------------------------------------------------------

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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineCursor {
    pub taken_at: i64,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelinePage {
    pub items: Vec<AssetSummary>,
    pub next: Option<TimelineCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExifField {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssetDetail {
    pub asset: AssetSummary,
    pub album_name: Option<String>,
    pub album_path: Option<String>,
    pub exif: Vec<ExifField>,
    pub download_name: String,
}
