//! Upload queue entries, progress and state events.

use super::*;

// ---------------------------------------------------------------------------
// Upload (v1 manual; durable queue and session snapshots)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum UploadState {
    Queued,
    CreatingSession,
    Uploading,
    Completing,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadTask {
    pub id: UploadTaskId,
    pub local_asset_id: String,
    pub name: String,
    pub dest_uri: String,
    pub dest_name: Option<String>,
    pub state: UploadState,
    pub uploaded: i64,
    pub total: i64,
    pub speed: i64,
    pub eta_seconds: Option<u32>,
    pub attempts: u32,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EnqueueUploadsParams {
    pub local_ids: Vec<String>,
    /// Empty means "Library Root / Inbox".
    #[serde(default)]
    pub dest_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadTaskParams {
    pub task_id: UploadTaskId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadProgressEvent {
    pub task_id: UploadTaskId,
    pub uploaded: i64,
    pub total: i64,
    pub speed: i64,
    pub eta_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadStateEvent {
    pub task_id: UploadTaskId,
    pub state: UploadState,
    pub error: Option<AppError>,
}
