//! Parameters the upload queue accepts, and the events it reports.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;
use crate::store::{AlbumId, LocalAssetId, UploadState, UploadTaskId};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EnqueueUploadsParams {
    pub local_ids: Vec<LocalAssetId>,
    /// The album (remote folder) every file of this batch lands in.
    pub album_id: AlbumId,
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
