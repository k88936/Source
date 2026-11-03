//! Library root and remote folder listing.

use super::*;

impl<C, R, M> AppCore<C, R, M>
where
    C: FolderStore + ServerStore,
    R: CloudrevePort,
{
    pub async fn list_remote_folders(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<crate::dto::RemoteFolderPage, AppError> {
        let page = self.cloud.list_entries(uri, page_token).await?;
        let parent = page.parent.as_ref().map(folder_ref);
        let folders = page
            .entries
            .into_iter()
            .filter(|e| e.is_folder)
            .map(|e| folder_ref(&e))
            .collect();
        Ok(crate::dto::RemoteFolderPage {
            folders,
            parent,
            next: page.next,
        })
    }

    pub async fn set_library_root(&self, uri: &str) -> Result<FolderRef, AppError> {
        let info = self.cloud.folder_info(uri).await?;
        let folder = folder_ref(&info);
        self.catalog
            .upsert_folder(&FolderRecord {
                id: folder.id.0.clone(),
                parent_id: None,
                name: folder.name.clone(),
                uri: folder.uri.clone(),
                path: folder.path.clone(),
                updated_at: info.updated_at,
                is_album: false,
                cover_asset_id: info.metadata.get(COVER_KEY).cloned(),
                description: info.metadata.get(DESCRIPTION_KEY).cloned(),
            })
            .await?;
        self.remember_library_root(&folder).await?;
        Ok(folder)
    }

    /// Point the stored root and the live session at `folder`.
    ///
    /// A full index re-derives the root's id from the remote, so the stored
    /// root is refreshed whenever the server hands out a different one.
    pub(crate) async fn remember_library_root(&self, folder: &FolderRef) -> Result<(), AppError> {
        self.catalog.set_library_root(folder).await?;
        let updated = {
            let mut guard = self.session.lock().unwrap();
            guard.as_mut().map(|session| {
                session.library_root = Some(folder.clone());
                session.clone()
            })
        };
        if let Some(session) = updated {
            self.cache_session(&session).await;
        }
        Ok(())
    }
}
