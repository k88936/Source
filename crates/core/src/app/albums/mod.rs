//! Albums: the remote folders a user added, and the local albums syncing into
//! them.
//!
//! An album is a folder, not a tree. Its assets are the folder's direct media
//! files, so refreshing an album is one listing one level deep: no recursion, no
//! sub-folders and no non-media entries.

mod dto;

use std::sync::Arc;

use crate::app::JobKind;
use crate::app::jobs::{Ctx, JobId, JobOutcome, JobPriority, Jobs};
use tokio_util::sync::CancellationToken;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::mapping::{COVER_KEY, DESCRIPTION_KEY, asset_record, entry_media_type};
use crate::media::{
    LocalAsset, LocalAssetPage, LocalAssetPageParams, LocalBrowsePage, LocalMediaPort, MediaError,
};
use crate::remote::{
    ListError, MediaFetchError, MetadataError, MoveError, RemoteError, RemoteStorage,
};
use crate::store::{
    AlbumId, AlbumMetadata, AlbumStore, AssetRecord, AssetStore, AssetSummary, CatalogError,
    FolderId, FolderRecord, FolderStore, LocalAlbumRecord, LocalStore, MediaType, SYNC_DONE,
    SYNC_FAILED, TimelineStore,
};
use crate::task::{Attempt, Retry, Step, attempt};
use crate::time::now_ms;

use super::{clamp_limit, folder_record, folder_ref};

pub use dto::*;

/// What an album operation can fail with.
///
/// The server-side causes stay typed: a folder that is gone and a server that
/// cannot be reached are different things to the user, and only the caller can
/// decide which one is worth retrying.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum AlbumError {
    /// The index could not answer.
    Catalog { cause: CatalogError },
    /// The album row the caller points at is gone.
    AlbumMissing { album: String },
    /// The asset row the caller points at is gone.
    AssetMissing { asset: String },
    /// The local album row the caller points at is gone.
    LocalAlbumMissing { uri: String },
    /// The URI is a file, not a folder.
    NotAFolder,
    /// Listing the folder failed.
    Listing { album: String, cause: ListError },
    /// The server refused the album's metadata patch.
    Metadata { album: String, cause: MetadataError },
    /// Moving files into the album failed.
    Move { cause: MoveError },
    /// The server moved a different number of files than asked.
    MoveMismatch,
    /// The device library failed.
    LocalMedia { cause: MediaError },
    /// Reading one album's asset left the index.
    Thumb {
        asset: String,
        cause: MediaFetchError,
    },
}

impl AlbumError {
    fn catalog(cause: CatalogError) -> Self {
        AlbumError::Catalog { cause }
    }

    /// Whether another listing could plausibly succeed.
    ///
    /// A refusal is the server's final answer; only a request that never got
    /// one is worth repeating.
    fn is_transient(&self) -> bool {
        matches!(
            self,
            AlbumError::Listing {
                cause: ListError::Transport {
                    cause: RemoteError::Unreachable { .. }
                },
                ..
            }
        )
    }

    /// Whether the folder itself is no longer reachable.
    fn ends_album(&self) -> bool {
        matches!(
            self,
            AlbumError::Listing {
                cause: ListError::FolderGone { .. },
                ..
            }
        )
    }
}

/// How a listing pass ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Listing {
    Complete,
    Stopped,
}

pub struct AlbumsService<R, C, M> {
    cloud: R,
    catalog: C,
    local: M,
    jobs: Jobs,
}

impl<R, C, M> AlbumsService<R, C, M> {
    pub fn new(cloud: R, catalog: C, local: M, jobs: Jobs) -> Self {
        Self {
            cloud,
            catalog,
            local,
            jobs,
        }
    }
}

impl<R, C, M> AlbumsService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AlbumStore + AssetStore + FolderStore + TimelineStore + LocalStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    pub async fn list_albums(&self) -> Result<Vec<AlbumMetadata>, AlbumError> {
        self.list_albums_impl().await
    }

    pub async fn get_album(&self, album_id: &str) -> Result<Option<AlbumMetadata>, AlbumError> {
        self.get_album_impl(album_id).await
    }

    /// Add a folder as an album and list it once, so it is never shown empty
    /// while the server has media in it.
    pub async fn add_album(&self, params: AddAlbumParams) -> Result<AlbumMetadata, AlbumError> {
        self.add_album_impl(params).await
    }

    /// Forget an album. The folder on the server is left alone, and every local
    /// album that was syncing into it goes with the row.
    pub async fn remove_album(&self, album_id: &str) -> Result<(), AlbumError> {
        self.remove_album_impl(album_id).await
    }

    /// List one album's direct media and make the catalog match.
    ///
    /// A folder that was deleted or renamed on the server keeps its row and its
    /// assets: the failure is recorded on the album so the UI can offer to point
    /// it somewhere else, instead of the album silently disappearing.
    pub async fn refresh_album(&self, album_id: &str) -> Result<AlbumMetadata, AlbumError> {
        self.refresh_album_impl(album_id).await
    }

    /// Queue one listing job per album, and return how many were queued.
    ///
    /// Nothing here waits: every album ends on its own row and disappears from
    /// the queue when it is done. An album already queued or listing is not
    /// queued again, so a second press while a refresh is running only picks up
    /// what finished meanwhile.
    pub async fn refresh_all(self: &Arc<Self>) -> Result<usize, AlbumError> {
        self.refresh_all_impl().await
    }

    /// Stop the refresh: what is still waiting is dropped, what is listing ends
    /// after the page it is on.
    pub fn stop_refresh(&self) {
        self.stop_refresh_impl()
    }

    pub async fn set_album_cover(
        &self,
        album_id: &str,
        asset_id: &str,
    ) -> Result<AlbumMetadata, AlbumError> {
        self.set_album_cover_impl(album_id, asset_id).await
    }

    pub async fn set_album_description(
        &self,
        album_id: &str,
        description: &str,
    ) -> Result<AlbumMetadata, AlbumError> {
        self.set_album_description_impl(album_id, description).await
    }

    /// Move assets into an album.
    ///
    /// In Cloudreve an album is a folder, so this is a real move: the files
    /// leave their current folder. The catalog rows are re-pointed only after
    /// the remote move succeeded.
    pub async fn move_assets_to_album(
        &self,
        asset_ids: &[String],
        album_id: &str,
    ) -> Result<Vec<AssetSummary>, AlbumError> {
        self.move_assets_to_album_impl(asset_ids, album_id).await
    }

    // ---------------------------------------------------------------------
    // Local albums
    // ---------------------------------------------------------------------

    pub async fn local_album_list(&self) -> Result<Vec<LocalAlbumMetadata>, AlbumError> {
        self.local_album_list_impl().await
    }

    /// One step of the device library browser the picker is built on.
    pub async fn browse_local(&self, parent: Option<&str>) -> Result<LocalBrowsePage, AlbumError> {
        self.browse_local_impl(parent).await
    }

    /// The direct media files of one local album, newest first.
    pub async fn get_local_asset_page(
        &self,
        params: LocalAssetPageParams,
    ) -> Result<LocalAssetPage, AlbumError> {
        self.get_local_asset_page_impl(params).await
    }

    pub async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AlbumError> {
        self.get_local_asset_impl(id).await
    }

    /// Register a device album - a directory, or a phone library bucket - as
    /// the source of one album.
    ///
    /// A local album only ever exists to sync somewhere, so the target album is
    /// part of the same write as the registration.
    pub async fn add_local_album(
        &self,
        params: AddLocalAlbumParams,
    ) -> Result<LocalAlbumMetadata, AlbumError> {
        self.add_local_album_impl(params).await
    }

    /// Stop syncing a device directory. Files already uploaded stay where they
    /// are, and the album is untouched.
    pub async fn remove_local_album(&self, id: &str) -> Result<(), AlbumError> {
        self.remove_local_album_impl(id).await
    }
}

// ---------------------------------------------------------------------
// Use case implementations
// ---------------------------------------------------------------------

impl<R, C, M> AlbumsService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AlbumStore + AssetStore + FolderStore + TimelineStore + LocalStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    async fn list_albums_impl(&self) -> Result<Vec<AlbumMetadata>, AlbumError> {
        self.catalog
            .list_albums()
            .await
            .map_err(AlbumError::catalog)
    }

    async fn get_album_impl(&self, album_id: &str) -> Result<Option<AlbumMetadata>, AlbumError> {
        self.catalog
            .get_album(album_id)
            .await
            .map_err(AlbumError::catalog)
    }

    async fn add_album_impl(&self, params: AddAlbumParams) -> Result<AlbumMetadata, AlbumError> {
        let info = self
            .cloud
            .get_folder_info(&params.uri)
            .await
            .map_err(|cause| AlbumError::Listing {
                album: params.uri.clone(),
                cause,
            })?;
        if !info.is_folder {
            return Err(AlbumError::NotAFolder);
        }
        let folder = folder_ref(&info);
        self.catalog
            .save_album(&folder_record(&folder, &info))
            .await
            .map_err(AlbumError::catalog)?;
        self.refresh_once(&folder.id.0).await
    }

    async fn remove_album_impl(&self, album_id: &str) -> Result<(), AlbumError> {
        self.catalog
            .delete_album(album_id)
            .await
            .map_err(AlbumError::catalog)
    }

    async fn refresh_album_impl(&self, album_id: &str) -> Result<AlbumMetadata, AlbumError> {
        self.refresh_once(album_id).await
    }

    async fn refresh_all_impl(self: &Arc<Self>) -> Result<usize, AlbumError> {
        let albums = self
            .catalog
            .list_albums()
            .await
            .map_err(AlbumError::catalog)?;
        for album in &albums {
            let service = Arc::clone(self);
            let album_id = album.id.0.clone();
            self.jobs.submit(
                JobKind::Index,
                JobId::from(album_id.clone()),
                JobPriority::RealTime,
                move |ctx| {
                    let albums = Arc::clone(&service);
                    let album_id = album_id.clone();
                    async move { albums.refresh_listing(&album_id, &ctx).await }
                },
            );
        }
        Ok(albums.len())
    }

    fn stop_refresh_impl(&self) {
        self.jobs.cancel(JobKind::Index);
    }

    async fn set_album_cover_impl(
        &self,
        album_id: &str,
        asset_id: &str,
    ) -> Result<AlbumMetadata, AlbumError> {
        let folder = self
            .catalog
            .get_folder(album_id)
            .await
            .map_err(AlbumError::catalog)?
            .ok_or_else(|| AlbumError::AlbumMissing {
                album: album_id.to_string(),
            })?;
        let patch = (COVER_KEY.to_string(), Some(asset_id.to_string()));
        self.cloud
            .patch_metadata(&folder.uri, &[patch])
            .await
            .map_err(|cause| AlbumError::Metadata {
                album: album_id.to_string(),
                cause,
            })?;
        self.catalog
            .set_album_cover(album_id, Some(asset_id))
            .await
            .map_err(AlbumError::catalog)?;
        self.album_metadata(album_id).await
    }

    async fn set_album_description_impl(
        &self,
        album_id: &str,
        description: &str,
    ) -> Result<AlbumMetadata, AlbumError> {
        let folder = self
            .catalog
            .get_folder(album_id)
            .await
            .map_err(AlbumError::catalog)?
            .ok_or_else(|| AlbumError::AlbumMissing {
                album: album_id.to_string(),
            })?;
        let value = if description.trim().is_empty() {
            None
        } else {
            Some(description.to_string())
        };
        self.cloud
            .patch_metadata(&folder.uri, &[(DESCRIPTION_KEY.to_string(), value.clone())])
            .await
            .map_err(|cause| AlbumError::Metadata {
                album: album_id.to_string(),
                cause,
            })?;
        self.catalog
            .set_album_description(album_id, value.as_deref())
            .await
            .map_err(AlbumError::catalog)?;
        self.album_metadata(album_id).await
    }

    async fn move_assets_to_album_impl(
        &self,
        asset_ids: &[String],
        album_id: &str,
    ) -> Result<Vec<AssetSummary>, AlbumError> {
        if asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let folder = self
            .catalog
            .get_folder(album_id)
            .await
            .map_err(AlbumError::catalog)?
            .ok_or_else(|| AlbumError::AlbumMissing {
                album: album_id.to_string(),
            })?;
        let mut records = Vec::with_capacity(asset_ids.len());
        for id in asset_ids {
            let record = self
                .catalog
                .get_asset(id)
                .await
                .map_err(AlbumError::catalog)?
                .ok_or_else(|| AlbumError::AssetMissing { asset: id.clone() })?;
            records.push(record);
        }
        let uris: Vec<String> = records.iter().map(|r| r.uri.clone()).collect();
        let new_uris = self
            .cloud
            .move_assets(&uris, &folder.uri)
            .await
            .map_err(|cause| AlbumError::Move { cause })?;
        if new_uris.len() != records.len() {
            return Err(AlbumError::MoveMismatch);
        }

        let album_id = FolderId::new(folder.id);
        let mut moved = Vec::with_capacity(records.len());
        for (record, new_uri) in records.into_iter().zip(new_uris) {
            let asset_id = record.summary.id.0.clone();
            self.catalog
                .move_asset(&asset_id, &album_id.0, &new_uri)
                .await
                .map_err(AlbumError::catalog)?;
            moved.push(AssetSummary {
                folder_id: album_id.clone(),
                ..record.summary
            });
        }
        Ok(moved)
    }

    async fn local_album_list_impl(&self) -> Result<Vec<LocalAlbumMetadata>, AlbumError> {
        let albums = self
            .catalog
            .list_local_albums()
            .await
            .map_err(AlbumError::catalog)?;
        let mut metadata = Vec::with_capacity(albums.len());
        for album in albums {
            metadata.push(self.local_album_to_metadata(album).await?);
        }
        Ok(metadata)
    }

    async fn browse_local_impl(&self, parent: Option<&str>) -> Result<LocalBrowsePage, AlbumError> {
        self.local
            .browse(parent)
            .await
            .map_err(|cause| AlbumError::LocalMedia { cause })
    }

    async fn get_local_asset_page_impl(
        &self,
        params: LocalAssetPageParams,
    ) -> Result<LocalAssetPage, AlbumError> {
        self.local
            .list_assets(
                &params.album_uri,
                params.cursor.as_deref(),
                clamp_limit(params.limit),
            )
            .await
            .map_err(|cause| AlbumError::LocalMedia { cause })
    }

    async fn get_local_asset_impl(&self, id: &str) -> Result<Option<LocalAsset>, AlbumError> {
        self.local
            .get_asset(id)
            .await
            .map_err(|cause| AlbumError::LocalMedia { cause })
    }

    async fn add_local_album_impl(
        &self,
        params: AddLocalAlbumParams,
    ) -> Result<LocalAlbumMetadata, AlbumError> {
        let album = self
            .catalog
            .get_album(&params.album_id.0)
            .await
            .map_err(AlbumError::catalog)?
            .ok_or_else(|| AlbumError::AlbumMissing {
                album: params.album_id.0.clone(),
            })?;
        self.catalog
            .add_local_album(&params.uri, &params.name, &album.id.0)
            .await
            .map_err(AlbumError::catalog)?;
        let record = self
            .catalog
            .get_local_album(&params.uri)
            .await
            .map_err(AlbumError::catalog)?
            .ok_or_else(|| AlbumError::LocalAlbumMissing {
                uri: params.uri.clone(),
            })?;
        self.local_album_to_metadata(record).await
    }

    async fn remove_local_album_impl(&self, id: &str) -> Result<(), AlbumError> {
        self.catalog
            .remove_local_album(id)
            .await
            .map_err(AlbumError::catalog)
    }

    async fn local_album_to_metadata(
        &self,
        album: LocalAlbumRecord,
    ) -> Result<LocalAlbumMetadata, AlbumError> {
        let asset_count = self
            .local
            .count_assets(&album.uri)
            .await
            .map_err(|cause| AlbumError::LocalMedia { cause })?;
        let covers = self
            .local
            .list_assets(&album.uri, None, 4)
            .await
            .map_err(|cause| AlbumError::LocalMedia { cause })?;
        Ok(LocalAlbumMetadata {
            id: album.id,
            uri: album.uri,
            name: album.name,
            album_id: AlbumId(album.album_id),
            album_name: album.album_name,
            asset_count,
            cover_uris: covers.items.into_iter().map(|asset| asset.uri).collect(),
        })
    }

    async fn album_metadata(&self, album_id: &str) -> Result<AlbumMetadata, AlbumError> {
        self.catalog
            .get_album(album_id)
            .await
            .map_err(AlbumError::catalog)?
            .ok_or_else(|| AlbumError::AlbumMissing {
                album: album_id.to_string(),
            })
    }

    /// One interactive listing: no job, so it reports no progress and stops
    /// only when the caller drops the future.
    async fn refresh_once(&self, album_id: &str) -> Result<AlbumMetadata, AlbumError> {
        let cancel = CancellationToken::new();
        self.refresh(album_id, &cancel, None).await
    }

    /// One album as a job: list it page by page, one step per page.
    async fn refresh_listing(
        &self,
        album_id: &str,
        ctx: &Ctx<JobKind>,
    ) -> Result<JobOutcome, AlbumError> {
        let ran = ctx
            .run_paused(|| {
                attempt(Retry::default(), ctx.cancel(), |_| async {
                    match self.refresh(album_id, ctx.cancel(), Some(ctx)).await {
                        Ok(_) => Step::Done(()),
                        Err(error) if error.is_transient() => Step::Retry(error),
                        Err(error) => Step::Failed(error),
                    }
                })
            })
            .await;
        match ran.value {
            Attempt::Done(_) => Ok(JobOutcome::Done),
            Attempt::Stopped if ran.paused => Ok(JobOutcome::Paused),
            Attempt::Stopped => Ok(JobOutcome::Done),
            Attempt::Failed(error) => Err(error),
        }
    }

    /// List one album, stopping between listing pages when `cancel` fires.
    ///
    /// `ctx` is the job listing this album, when there is one: it is where the
    /// page count goes. The interactive refresh has no job and reports nowhere.
    async fn refresh(
        &self,
        album_id: &str,
        cancel: &CancellationToken,
        ctx: Option<&Ctx<JobKind>>,
    ) -> Result<AlbumMetadata, AlbumError> {
        let album = self
            .catalog
            .get_folder(album_id)
            .await
            .map_err(AlbumError::catalog)?
            .ok_or_else(|| AlbumError::AlbumMissing {
                album: album_id.to_string(),
            })?;
        match self.sync_album(&album, cancel, ctx).await {
            Ok(Listing::Stopped) => return self.album_metadata(album_id).await,
            Ok(Listing::Complete) => {}
            Err(error) => {
                // Only a folder the server no longer gives us is a lasting
                // failure; a dropped connection leaves the last state alone so
                // the banner does not fire for a blip.
                if error.ends_album() {
                    self.catalog
                        .mark_folder(&album.id, SYNC_FAILED, None, now_ms())
                        .await
                        .map_err(AlbumError::catalog)?;
                }
                return Err(error);
            }
        }
        self.album_metadata(album_id).await
    }

    async fn sync_album(
        &self,
        album: &FolderRecord,
        cancel: &CancellationToken,
        ctx: Option<&Ctx<JobKind>>,
    ) -> Result<Listing, AlbumError> {
        let folder_id = FolderId::new(album.id.clone());
        // Every id the listing returned, for the prune below. The rows
        // themselves go in page by page, so a big album never sits in memory
        // whole and a page boundary is a place the pass can be stopped.
        let mut listed = Vec::new();
        let mut token: Option<String> = None;
        let mut remote_updated_at = 0;
        loop {
            if cancel.is_cancelled() {
                // A stop is the loop ending, not a failure of the album.
                return Ok(Listing::Stopped);
            }
            let page = self
                .cloud
                .list_entries(&album.uri, token.as_deref())
                .await
                .map_err(|cause| AlbumError::Listing {
                    album: album.id.clone(),
                    cause,
                })?;
            let records: Vec<AssetRecord> = page
                .entries
                .iter()
                .filter(|entry| is_media_entry(entry))
                .map(|entry| {
                    remote_updated_at = remote_updated_at.max(entry.updated_at);
                    asset_record(entry, &folder_id, None)
                })
                .collect();
            listed.extend(records.iter().map(|record| record.summary.id.0.clone()));
            self.catalog
                .upsert_album_assets(&records)
                .await
                .map_err(AlbumError::catalog)?;
            // One page is one step, so a big album moves while it is listed
            // instead of jumping from nothing to done.
            if let Some(ctx) = ctx {
                ctx.advance(1);
            }
            token = page.next;
            if token.is_none() {
                break;
            }
        }
        // Only a listing that ran to the end can tell a removed file from a
        // page this pass has not reached.
        self.catalog
            .prune_album_assets(&album.id, &listed)
            .await
            .map_err(AlbumError::catalog)?;
        self.catalog
            .mark_folder(&album.id, SYNC_DONE, Some(remote_updated_at), now_ms())
            .await
            .map_err(AlbumError::catalog)?;
        Ok(Listing::Complete)
    }
}

fn is_media_entry(entry: &crate::remote::RemoteEntry) -> bool {
    !entry.is_folder && matches!(entry_media_type(entry), MediaType::Image | MediaType::Video)
}
