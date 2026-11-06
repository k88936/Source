//! Albums: the remote folders a user added, and the local albums syncing into
//! them.
//!
//! An album is a folder, not a tree. Its assets are the folder's direct media
//! files, so refreshing an album is one listing one level deep: no recursion, no
//! sub-folders and no non-media entries.

mod dto;

use std::sync::Arc;

use source_jobs::{Ctx, JobId, JobKind, JobOutcome, JobPriority, Jobs};
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, ErrorCode};
use crate::mapping::{COVER_KEY, DESCRIPTION_KEY, asset_record, entry_media_type};
use crate::media::{
    LocalAsset, LocalAssetPage, LocalAssetPageParams, LocalBrowsePage, LocalMediaPort,
};
use crate::remote::RemoteStorage;
use crate::store::{
    AlbumId, AlbumStore, AlbumSummary, AssetRecord, AssetStore, AssetSummary, FolderId,
    FolderRecord, FolderStore, LocalAlbumRecord, LocalStore, MediaType, SYNC_DONE, SYNC_FAILED,
    TimelinePage, TimelineStore,
};
use crate::task::{Retry, attempt};
use crate::time::now_ms;

use super::{clamp_limit, folder_record, folder_ref};

pub use dto::*;

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

/// One album listing is one job. The queue only compares the key, so what a
/// refresh is stays here, and the stop button reaches every album through it.
pub const INDEX_JOB: JobKind = JobKind::new("index");

impl<R, C, M> AlbumsService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AlbumStore + AssetStore + FolderStore + TimelineStore + LocalStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    pub async fn list_albums(&self) -> Result<Vec<AlbumSummary>, AppError> {
        self.list_albums_impl().await
    }

    pub async fn get_album(&self, album_id: &str) -> Result<Option<AlbumSummary>, AppError> {
        self.get_album_impl(album_id).await
    }

    pub async fn get_album_page(&self, params: AlbumPageParams) -> Result<TimelinePage, AppError> {
        self.get_album_page_impl(params).await
    }

    /// Add a folder as an album and list it once, so it is never shown empty
    /// while the server has media in it.
    pub async fn add_album(&self, params: AddAlbumParams) -> Result<AlbumSummary, AppError> {
        self.add_album_impl(params).await
    }

    /// Forget an album. The folder on the server is left alone, and every local
    /// album that was syncing into it goes with the row.
    pub async fn remove_album(&self, album_id: &str) -> Result<(), AppError> {
        self.remove_album_impl(album_id).await
    }

    /// List one album's direct media and make the catalog match.
    ///
    /// A folder that was deleted or renamed on the server keeps its row and its
    /// assets: the failure is recorded on the album so the UI can offer to point
    /// it somewhere else, instead of the album silently disappearing.
    pub async fn refresh_album(&self, album_id: &str) -> Result<AlbumSummary, AppError> {
        self.refresh_album_impl(album_id).await
    }

    /// Queue one listing job per album, and return how many were queued.
    ///
    /// Nothing here waits: every album ends on its own row and disappears from
    /// the queue when it is done. An album already queued or listing is not
    /// queued again, so a second press while a refresh is running only picks up
    /// what finished meanwhile.
    pub async fn refresh_all(self: &Arc<Self>) -> Result<usize, AppError> {
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
    ) -> Result<AlbumSummary, AppError> {
        self.set_album_cover_impl(album_id, asset_id).await
    }

    pub async fn set_album_description(
        &self,
        album_id: &str,
        description: &str,
    ) -> Result<AlbumSummary, AppError> {
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
    ) -> Result<Vec<AssetSummary>, AppError> {
        self.move_assets_to_album_impl(asset_ids, album_id).await
    }

    // ---------------------------------------------------------------------
    // Local albums
    // ---------------------------------------------------------------------

    pub async fn local_album_list(&self) -> Result<Vec<LocalAlbumSummary>, AppError> {
        self.local_album_list_impl().await
    }

    /// One step of the device library browser the picker is built on.
    pub async fn browse_local(&self, parent: Option<&str>) -> Result<LocalBrowsePage, AppError> {
        self.browse_local_impl(parent).await
    }

    /// The direct media files of one local album, newest first.
    pub async fn get_local_asset_page(
        &self,
        params: LocalAssetPageParams,
    ) -> Result<LocalAssetPage, AppError> {
        self.get_local_asset_page_impl(params).await
    }

    pub async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
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
    ) -> Result<LocalAlbumSummary, AppError> {
        self.add_local_album_impl(params).await
    }

    /// Stop syncing a device directory. Files already uploaded stay where they
    /// are, and the album is untouched.
    pub async fn remove_local_album(&self, id: &str) -> Result<(), AppError> {
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
    async fn list_albums_impl(&self) -> Result<Vec<AlbumSummary>, AppError> {
        self.catalog.list_albums().await
    }

    async fn get_album_impl(&self, album_id: &str) -> Result<Option<AlbumSummary>, AppError> {
        self.catalog.get_album(album_id).await
    }

    async fn get_album_page_impl(&self, params: AlbumPageParams) -> Result<TimelinePage, AppError> {
        self.catalog
            .get_album_page(
                &params.album_id.0,
                params.cursor.as_ref(),
                clamp_limit(params.limit),
            )
            .await
    }

    async fn add_album_impl(&self, params: AddAlbumParams) -> Result<AlbumSummary, AppError> {
        let info = self.cloud.get_folder_info(&params.uri).await?;
        if !info.is_folder {
            return Err(AppError::internal("只能把文件夹添加为相册"));
        }
        let folder = folder_ref(&info);
        self.catalog
            .save_album(&folder_record(&folder, &info))
            .await?;
        self.refresh_once(&folder.id.0).await
    }

    async fn remove_album_impl(&self, album_id: &str) -> Result<(), AppError> {
        self.catalog.delete_album(album_id).await
    }

    async fn refresh_album_impl(&self, album_id: &str) -> Result<AlbumSummary, AppError> {
        self.refresh_once(album_id).await
    }

    async fn refresh_all_impl(self: &Arc<Self>) -> Result<usize, AppError> {
        let ids = self
            .catalog
            .list_albums()
            .await?
            .into_iter()
            .map(|album| album.id.0)
            .collect::<Vec<_>>();
        for id in &ids {
            let albums = Arc::clone(self);
            let album_id = id.clone();
            self.jobs.submit(
                INDEX_JOB,
                JobId::from(id.clone()),
                JobPriority::RealTime,
                move |ctx| {
                    let albums = Arc::clone(&albums);
                    let album_id = album_id.clone();
                    async move { albums.refresh_listing(&album_id, &ctx).await }
                },
            );
        }
        Ok(ids.len())
    }

    fn stop_refresh_impl(&self) {
        self.jobs.cancel(INDEX_JOB);
    }

    async fn set_album_cover_impl(
        &self,
        album_id: &str,
        asset_id: &str,
    ) -> Result<AlbumSummary, AppError> {
        let folder = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let patch = (COVER_KEY.to_string(), Some(asset_id.to_string()));
        self.cloud.patch_metadata(&folder.uri, &[patch]).await?;
        self.catalog
            .set_album_cover(album_id, Some(asset_id))
            .await?;
        self.catalog
            .get_album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    async fn set_album_description_impl(
        &self,
        album_id: &str,
        description: &str,
    ) -> Result<AlbumSummary, AppError> {
        let folder = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let value = if description.trim().is_empty() {
            None
        } else {
            Some(description.to_string())
        };
        self.cloud
            .patch_metadata(&folder.uri, &[(DESCRIPTION_KEY.to_string(), value.clone())])
            .await?;
        self.catalog
            .set_album_description(album_id, value.as_deref())
            .await?;
        self.catalog
            .get_album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    async fn move_assets_to_album_impl(
        &self,
        asset_ids: &[String],
        album_id: &str,
    ) -> Result<Vec<AssetSummary>, AppError> {
        if asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let folder = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let mut records = Vec::with_capacity(asset_ids.len());
        for id in asset_ids {
            let record = self
                .catalog
                .get_asset(id)
                .await?
                .ok_or_else(|| AppError::not_found("资产不存在"))?;
            records.push(record);
        }
        let uris: Vec<String> = records.iter().map(|r| r.uri.clone()).collect();
        let new_uris = self.cloud.move_assets(&uris, &folder.uri).await?;
        if new_uris.len() != records.len() {
            return Err(AppError::internal("移动结果与请求不一致"));
        }

        let album_id = FolderId::new(folder.id);
        let mut moved = Vec::with_capacity(records.len());
        for (record, new_uri) in records.into_iter().zip(new_uris) {
            let asset_id = record.summary.id.0.clone();
            self.catalog
                .move_asset(&asset_id, &album_id.0, &new_uri)
                .await?;
            moved.push(AssetSummary {
                folder_id: album_id.clone(),
                ..record.summary
            });
        }
        Ok(moved)
    }

    async fn local_album_list_impl(&self) -> Result<Vec<LocalAlbumSummary>, AppError> {
        let albums = self.catalog.list_local_albums().await?;
        let mut summaries = Vec::with_capacity(albums.len());
        for album in albums {
            summaries.push(self.local_album_to_summary(album).await?);
        }
        Ok(summaries)
    }

    async fn browse_local_impl(&self, parent: Option<&str>) -> Result<LocalBrowsePage, AppError> {
        self.local.browse(parent).await
    }

    async fn get_local_asset_page_impl(
        &self,
        params: LocalAssetPageParams,
    ) -> Result<LocalAssetPage, AppError> {
        self.local
            .list_assets(
                &params.album_uri,
                params.cursor.as_deref(),
                clamp_limit(params.limit),
            )
            .await
    }

    async fn get_local_asset_impl(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
        self.local.get_asset(id).await
    }

    async fn add_local_album_impl(
        &self,
        params: AddLocalAlbumParams,
    ) -> Result<LocalAlbumSummary, AppError> {
        let album = self
            .catalog
            .get_album(&params.album_id.0)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        self.catalog
            .add_local_album(&params.uri, &params.name, &album.id.0)
            .await?;
        let record = self
            .catalog
            .get_local_album(&params.uri)
            .await?
            .ok_or_else(|| AppError::not_found("本机相册不存在"))?;
        self.local_album_to_summary(record).await
    }

    async fn remove_local_album_impl(&self, id: &str) -> Result<(), AppError> {
        self.catalog.remove_local_album(id).await
    }

    async fn local_album_to_summary(
        &self,
        album: LocalAlbumRecord,
    ) -> Result<LocalAlbumSummary, AppError> {
        let asset_count = self.local.count_assets(&album.uri).await?;
        Ok(LocalAlbumSummary {
            id: album.id,
            uri: album.uri,
            name: album.name,
            album_id: AlbumId(album.album_id),
            album_name: album.album_name,
            asset_count,
        })
    }

    /// One interactive listing: no job, so it reports no progress and stops
    /// only when the caller drops the future.
    async fn refresh_once(&self, album_id: &str) -> Result<AlbumSummary, AppError> {
        let cancel = CancellationToken::new();
        self.refresh(album_id, &cancel, None).await
    }

    /// One album as a job: list it page by page, one step per page.
    async fn refresh_listing(&self, album_id: &str, ctx: &Ctx) -> Result<JobOutcome, AppError> {
        let ran = ctx
            .run_paused(|| {
                attempt(Retry::default(), ctx.cancel(), |_| {
                    self.refresh(album_id, ctx.cancel(), Some(ctx))
                })
            })
            .await;
        match ran.value {
            Ok(_) => Ok(JobOutcome::Done),
            Err(err) if err.code == ErrorCode::Cancelled && ran.paused => Ok(JobOutcome::Paused),
            Err(err) if err.code == ErrorCode::Cancelled => Ok(JobOutcome::Done),
            Err(err) => Err(err),
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
        ctx: Option<&Ctx>,
    ) -> Result<AlbumSummary, AppError> {
        let album = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        if let Err(err) = self.sync_album(&album, cancel, ctx).await {
            // A stop is not a failure of the album: leave its last state alone.
            if err.code != ErrorCode::Cancelled {
                self.catalog
                    .mark_folder(&album.id, SYNC_FAILED, Some(&err.message), None, now_ms())
                    .await?;
            }
            return Err(err);
        }
        self.catalog
            .get_album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    async fn sync_album(
        &self,
        album: &FolderRecord,
        cancel: &CancellationToken,
        ctx: Option<&Ctx>,
    ) -> Result<(), AppError> {
        let folder_id = FolderId::new(album.id.clone());
        // Every id the listing returned, for the prune below. The rows
        // themselves go in page by page, so a big album never sits in memory
        // whole and a page boundary is a place the pass can be stopped.
        let mut listed = Vec::new();
        let mut token: Option<String> = None;
        let mut remote_updated_at = 0;
        loop {
            if cancel.is_cancelled() {
                return Err(AppError::cancelled());
            }
            let page = self
                .cloud
                .list_entries(&album.uri, token.as_deref())
                .await?;
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
            self.catalog.upsert_album_assets(&records).await?;
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
        self.catalog.prune_album_assets(&album.id, &listed).await?;
        self.catalog
            .mark_folder(
                &album.id,
                SYNC_DONE,
                None,
                Some(remote_updated_at),
                now_ms(),
            )
            .await?;
        Ok(())
    }
}

fn is_media_entry(entry: &crate::remote::RemoteEntry) -> bool {
    !entry.is_folder && matches!(entry_media_type(entry), MediaType::Image | MediaType::Video)
}
