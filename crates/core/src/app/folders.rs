//! Picking a remote folder: the listing the album picker browses, and the
//! folders it can create on the way.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;
use crate::remote::RemoteStorage;
use crate::store::{FolderRef, FolderStore};

use super::{folder_record, folder_ref};

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

pub struct FoldersService<R, C> {
    cloud: R,
    catalog: C,
}

impl<R, C> FoldersService<R, C> {
    pub fn new(cloud: R, catalog: C) -> Self {
        Self { cloud, catalog }
    }
}

impl<R, C> FoldersService<R, C>
where
    R: RemoteStorage,
    C: FolderStore,
{
    /// One level of folders below `uri`, files excluded: the picker only ever
    /// selects a folder.
    pub async fn list_remote_folders(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteFolderPage, AppError> {
        let page = self.cloud.list_entries(uri, page_token).await?;
        let current = folder_ref(&page.current);
        let folders = page
            .entries
            .into_iter()
            .filter(|e| e.is_folder)
            .map(|e| folder_ref(&e))
            .collect();
        Ok(RemoteFolderPage {
            folders,
            current,
            next: page.next,
        })
    }

    /// Create a folder from inside the picker and hand back the selectable
    /// reference, so the user does not have to leave the app to add an album.
    pub async fn create_folder(&self, parent_uri: &str, name: &str) -> Result<FolderRef, AppError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::internal("文件夹名不能为空"));
        }
        let entry = self.cloud.create_folder(parent_uri, name).await?;
        let folder = folder_ref(&entry);
        self.catalog
            .upsert_folder(&folder_record(&folder, &entry))
            .await?;
        Ok(folder)
    }
}
