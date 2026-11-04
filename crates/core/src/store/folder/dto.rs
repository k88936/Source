//! Folder payloads: the album reference, the album record and its sync state.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::id_type;

id_type!(FolderId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FolderRef {
    pub id: FolderId,
    pub uri: String,
    pub name: String,
    /// Path relative to the user's root (`/Photos/2024` style).
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct FolderRecord {
    pub id: String,
    pub name: String,
    pub uri: String,
    /// Cloudreve's own path (`cloudreve://my/%E9%AA%8C%E8%AF%81`), kept as the
    /// server sent it; `display_path` is the form the UI reads.
    pub path: String,
    pub updated_at: i64,
    pub cover_asset_id: Option<String>,
    pub description: Option<String>,
}

/// `sync_state.status` of an album whose last listing succeeded.
pub const SYNC_DONE: &str = "done";

/// `sync_state.status` of an album whose last listing failed: the folder is
/// gone or unreadable, and the album has to be pointed somewhere else.
pub const SYNC_FAILED: &str = "failed";

#[derive(Debug, Clone)]
pub struct FolderSyncState {
    pub folder_id: String,
    pub status: String,
    pub remote_updated_at: Option<i64>,
    pub last_listed_at: Option<i64>,
    pub last_error: Option<String>,
}

impl FolderSyncState {
    pub fn is_done(&self) -> bool {
        self.status == SYNC_DONE
    }
}
