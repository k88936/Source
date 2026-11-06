//! Parameters the folder picker accepts, and the page it browses.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::FolderRef;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListRemoteFoldersParams {
    /// The folder to list; the drive root when absent.
    #[serde(default = "root_uri")]
    pub uri: String,
    pub page_token: Option<String>,
}

fn root_uri() -> String {
    "cloudreve://my".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RemoteFolderPage {
    pub folders: Vec<FolderRef>,
    /// Folder currently being browsed; the drive root has an empty name.
    pub current: FolderRef,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateFolderParams {
    /// The folder the new one is created in.
    pub parent_uri: String,
    pub name: String,
}
