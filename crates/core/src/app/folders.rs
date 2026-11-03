//! Library root and remote folder listing.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;
use crate::remote::RemoteStorage;
use crate::store::{AccountStore, FolderRef, FolderStore};

use super::{folder_ref, upsert_remote_folder};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListRemoteFoldersParams {
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
    /// Folder currently being browsed (absent for the user root).
    pub parent: Option<FolderRef>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetLibraryRootParams {
    pub uri: String,
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
    C: FolderStore + AccountStore,
{
    pub async fn list_remote_folders(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteFolderPage, AppError> {
        let page = self.cloud.list_entries(uri, page_token).await?;
        let parent = page.parent.as_ref().map(folder_ref);
        let folders = page
            .entries
            .into_iter()
            .filter(|e| e.is_folder)
            .map(|e| folder_ref(&e))
            .collect();
        Ok(RemoteFolderPage {
            folders,
            parent,
            next: page.next,
        })
    }

    pub async fn set_library_root(&self, uri: &str) -> Result<FolderRef, AppError> {
        let account = self
            .catalog
            .current_account()
            .await?
            .ok_or_else(|| AppError::internal("尚未选择服务器"))?;
        let info = self.cloud.folder_info(uri).await?;
        let folder = folder_ref(&info);
        upsert_remote_folder(&self.catalog, &folder, &info, None).await?;
        self.catalog
            .set_library_root(&account.base_url, &folder)
            .await?;
        Ok(folder)
    }
}
