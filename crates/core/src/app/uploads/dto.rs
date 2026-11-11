//! Parameters the upload queue accepts, the rows it reports, and the events it
//! emits.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::{AlbumId, LocalAssetId, UploadState, UploadTask, UploadTaskId};

use super::UploadError;

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

/// One row of the queue, with the failure of its last attempt read back into
/// the type the upload service owns.
///
/// The table only carries the bytes of that failure; parsing here is what keeps
/// the store free of the app's error vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct QueuedUpload {
    pub id: UploadTaskId,
    pub local_asset_id: LocalAssetId,
    /// The local album row that queued this file, when a mapping did.
    pub local_album_id: Option<String>,
    pub name: String,
    pub dest_uri: String,
    pub dest_name: Option<String>,
    pub state: UploadState,
    pub uploaded: i64,
    pub total: i64,
    pub speed: i64,
    pub eta_seconds: Option<u32>,
    pub attempts: u32,
    pub error: Option<UploadError>,
}

impl From<UploadTask> for QueuedUpload {
    fn from(row: UploadTask) -> Self {
        QueuedUpload {
            id: row.id,
            local_asset_id: row.local_asset_id,
            local_album_id: row.local_album_id,
            name: row.name,
            dest_uri: row.dest_uri,
            dest_name: row.dest_name,
            state: row.state,
            uploaded: row.uploaded,
            total: row.total,
            speed: row.speed,
            eta_seconds: row.eta_seconds,
            attempts: row.attempts,
            // A row written by an older build, or one whose failure no longer
            // deserializes, reads as "no failure": the state still says Failed,
            // which is what the card renders from.
            error: row
                .error
                .and_then(|raw| serde_json::from_str::<UploadError>(&raw).ok()),
        }
    }
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
    pub error: Option<UploadError>,
}
