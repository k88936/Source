//! Picking a remote folder: the listing the album picker browses, and the
//! folders it can create on the way.

mod dto;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::remote::{CreateFolderError, ListError, RemoteStorage};
use crate::store::{CatalogError, FolderRef, FolderStore};

use super::{folder_record, folder_ref};

pub use dto::*;

/// What picking a remote folder can fail with.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum FolderError {
    /// The folder being browsed is gone or the server could not be reached.
    Listing { uri: String, cause: ListError },
    /// Creating the folder itself was refused.
    Creation { cause: CreateFolderError },
    /// The name typed in was empty.
    EmptyName,
    /// The index could not record the new folder.
    Catalog { cause: CatalogError },
}

impl FolderError {
    fn catalog(cause: CatalogError) -> Self {
        FolderError::Catalog { cause }
    }
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
    ) -> Result<RemoteFolderPage, FolderError> {
        self.list_remote_folders_impl(uri, page_token).await
    }

    /// Create a folder from inside the picker and hand back the selectable
    /// reference, so the user does not have to leave the app to add an album.
    pub async fn create_folder(
        &self,
        parent_uri: &str,
        name: &str,
    ) -> Result<FolderRef, FolderError> {
        self.create_folder_impl(parent_uri, name).await
    }
}

// ---------------------------------------------------------------------
// Use case implementations
// ---------------------------------------------------------------------

impl<R, C> FoldersService<R, C>
where
    R: RemoteStorage,
    C: FolderStore,
{
    async fn list_remote_folders_impl(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteFolderPage, FolderError> {
        let page = self
            .cloud
            .list_entries(uri, page_token)
            .await
            .map_err(|cause| FolderError::Listing {
                uri: uri.to_string(),
                cause,
            })?;
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

    async fn create_folder_impl(
        &self,
        parent_uri: &str,
        name: &str,
    ) -> Result<FolderRef, FolderError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(FolderError::EmptyName);
        }
        let entry = self
            .cloud
            .create_folder(parent_uri, name)
            .await
            .map_err(|cause| FolderError::Creation { cause })?;
        let folder = folder_ref(&entry);
        self.catalog
            .upsert_folder(&folder_record(&folder, &entry))
            .await
            .map_err(FolderError::catalog)?;
        Ok(folder)
    }
}
