//! What crosses the remote contract: upload attempts, their progress, the
//! navigator entries the server returns, and the media bytes it streams back.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::store::UploadTaskId;

/// Receives one progress sample per uploader event.
pub type ProgressSink = Arc<dyn Fn(UploadProgress) + Send + Sync>;

#[derive(Debug, Clone, Copy, Default)]
pub struct UploadProgress {
    pub uploaded: i64,
    pub total: i64,
    pub speed: i64,
    pub eta_seconds: Option<u32>,
}

/// One upload attempt: what to send, where, and how to report it.
///
/// `task_id` doubles as the key of the resumable session, so a retry of the
/// same queue entry finds the chunks an earlier attempt already finished.
pub struct UploadRequest {
    pub task_id: UploadTaskId,
    pub source_uri: String,
    pub size: i64,
    pub dest_uri: String,
    pub name: String,
    pub mime: Option<String>,
    /// Raised by `cancel_upload`; uploaders stop between two chunks.
    pub cancel: CancellationToken,
    pub progress: ProgressSink,
}

/// A file or folder returned by Cloudreve's navigator.
#[derive(Debug, Clone)]
pub struct RemoteEntry {
    pub id: String,
    pub uri: String,
    pub name: String,
    pub path: String,
    pub is_folder: bool,
    pub size: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RemoteEntryPage {
    pub entries: Vec<RemoteEntry>,
    pub next: Option<String>,
    /// The folder this page was taken of.
    pub current: RemoteEntry,
}

/// One media response as the server sent it: status, body and the two headers
/// a range request needs to be replayed. `status` is passed through untouched
/// because `206`, `200` and errors are all legitimate answers to a range.
#[derive(Debug, Clone)]
pub struct MediaRange {
    pub status: u16,
    pub body: Vec<u8>,
    pub content_range: Option<String>,
    pub content_type: Option<String>,
}

impl MediaRange {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Capacity {
    pub total: i64,
    pub used: i64,
    pub storage_pack_total: i64,
}
