//! What crosses the remote contract: upload attempts, their progress, and the
//! navigator entries the server returns.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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
/// `task_id` doubles as the key of the resumable journal, so a retry of the
/// same queue entry finds the chunks an earlier attempt already finished.
pub struct UploadRequest {
    pub task_id: UploadTaskId,
    pub source_uri: String,
    pub size: i64,
    pub dest_uri: String,
    pub name: String,
    pub mime: Option<String>,
    /// Raised by `cancel_upload`; uploaders stop between two chunks.
    pub cancel: Arc<AtomicBool>,
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
    pub parent: Option<RemoteEntry>,
}
