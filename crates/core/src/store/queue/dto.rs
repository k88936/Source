//! Upload queue payloads: the queue entry and the state machine it moves
//! through.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;
use crate::store::id_type;

id_type!(UploadTaskId);

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
