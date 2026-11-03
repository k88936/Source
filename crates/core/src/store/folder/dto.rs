//! Folder payloads: the root reference, the mirrored folder record and its
//! sync state.

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
    pub parent_id: Option<String>,
    pub name: String,
    pub uri: String,
    pub path: String,
    pub updated_at: i64,
    pub is_album: bool,
    pub cover_asset_id: Option<String>,
    pub description: Option<String>,
}

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
        self.status == "done"
    }
}
