//! Library indexing.

use super::*;

// ---------------------------------------------------------------------------
// Indexing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IndexPhase {
    Idle,
    Scanning,
    Paused,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IndexStatus {
    pub phase: IndexPhase,
    pub folders_done: u32,
    pub folders_total: u32,
    pub assets_seen: i64,
    pub started_at: Option<i64>,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartIndexParams {
    #[serde(default)]
    pub full: bool,
}
