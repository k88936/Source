//! Sync: pushing the files of the registered local albums into their albums.
//!
//! A local album is a device directory the user pointed at one album. Scanning
//! one directory is one job: it walks the directory page by page, writes every
//! file the album does not already hold into the upload queue, and counts one
//! step per page. Everything after the row belongs to the upload queue: every
//! row becomes an upload job of its own, with its own retries and its own
//! session resume. A scan only decides what to queue.

use std::collections::HashSet;
use std::sync::Arc;

use source_jobs::{Ctx, JobId, JobKind, JobOutcome, JobPriority, Jobs};

use crate::media::{LocalAsset, LocalMediaPort, MediaError};
use crate::remote::RemoteStorage;
use crate::store::{
    AlbumId, AlbumStore, AssetStore, CatalogError, FolderStore, LocalAlbumRecord, LocalStore,
    UploadQueueStore,
};

use super::uploads::{UploadError, UploadsService};

/// How many files one page of a device directory lists.
const LOCAL_PAGE: u32 = 200;

/// One directory scan is one job. The queue only compares the key, so what a
/// sync is stays here, and the stop button reaches every directory through it.
pub const SYNC_JOB: JobKind = JobKind::new("sync");

/// What a scan can fail with.
///
/// A scan only reads the device and writes the queue, so its failures are those
/// two: a directory the platform refuses, and a queue write that did not land.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum SyncError {
    /// The index could not answer.
    Catalog { cause: CatalogError },
    /// The device library could not list this directory.
    DirectoryUnreadable { uri: String, cause: MediaError },
    /// The file could not be written into the upload queue.
    QueueFailed { cause: UploadError },
}

pub struct SyncService<R, C, M> {
    catalog: C,
    local: M,
    uploads: Arc<UploadsService<R, C, M>>,
    jobs: Jobs,
}

impl<R, C, M> SyncService<R, C, M> {
    pub fn new(catalog: C, local: M, uploads: Arc<UploadsService<R, C, M>>, jobs: Jobs) -> Self {
        Self {
            catalog,
            local,
            uploads,
            jobs,
        }
    }
}

impl<R, C, M> SyncService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AlbumStore + AssetStore + FolderStore + LocalStore + UploadQueueStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    /// Queue one scan per registered local album, and return how many.
    ///
    /// Nothing here waits, and nothing here counts files: a directory is walked
    /// by its job, one page at a time. The files it queues become upload jobs in
    /// the same queue, so a scan and the uploads it started share one budget
    /// instead of each holding its own.
    pub async fn sync_all(self: &Arc<Self>) -> Result<usize, SyncError> {
        self.sync_all_impl().await
    }

    /// Stop the scan: what is still waiting is dropped, what is walking ends
    /// after the page it is on. Files already queued keep uploading.
    pub fn stop(&self) {
        self.stop_impl()
    }
}

// ---------------------------------------------------------------------
// Use case implementations
// ---------------------------------------------------------------------

impl<R, C, M> SyncService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AlbumStore + AssetStore + FolderStore + LocalStore + UploadQueueStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    async fn sync_all_impl(self: &Arc<Self>) -> Result<usize, SyncError> {
        let albums = self
            .catalog
            .list_local_albums()
            .await
            .map_err(|cause| SyncError::Catalog { cause })?;
        for album in &albums {
            let sync = Arc::clone(self);
            let album = album.clone();
            self.jobs.submit(
                SYNC_JOB,
                JobId::from(album.id.clone()),
                JobPriority::RealTime,
                move |ctx| {
                    let sync = Arc::clone(&sync);
                    let album = album.clone();
                    async move { sync.scan_album(&album, &ctx).await }
                },
            );
        }
        Ok(albums.len())
    }

    fn stop_impl(&self) {
        self.jobs.cancel(SYNC_JOB);
    }

    /// Walk one directory: queue every file the album does not hold yet.
    ///
    /// A file is handled when it is already in the album or already sitting in
    /// the queue; only a directory that cannot be walked fails.
    async fn scan_album(
        &self,
        album: &LocalAlbumRecord,
        ctx: &Ctx,
    ) -> Result<JobOutcome, SyncError> {
        let Some(folder) = self
            .catalog
            .get_folder(&album.album_id)
            .await
            .map_err(|cause| SyncError::Catalog { cause })?
        else {
            // The album is gone, and the local row pointing at it goes with it.
            return Ok(JobOutcome::Done);
        };
        let names = self
            .catalog
            .list_album_asset_names(&folder.id)
            .await
            .map_err(|cause| SyncError::Catalog { cause })?;
        // Matched by name, so a directory of tens of thousands of files does
        // not turn every page into a scan of everything the album holds.
        let held: HashSet<&str> = names.iter().map(String::as_str).collect();
        let album_id = AlbumId(album.album_id.clone());

        let mut cursor: Option<String> = None;
        loop {
            if ctx.pause_token().requested() {
                return Ok(JobOutcome::Paused);
            }
            if ctx.cancel().is_cancelled() {
                break;
            }
            let page = self
                .local
                .list_assets(&album.uri, cursor.as_deref(), LOCAL_PAGE)
                .await
                .map_err(|cause| SyncError::DirectoryUnreadable {
                    uri: album.uri.clone(),
                    cause,
                })?;
            let missing: Vec<LocalAsset> = page
                .items
                .into_iter()
                .filter(|asset| !held.contains(asset.name.as_str()))
                .collect();
            // The page goes straight into the queue: a scan and the uploads it
            // starts share one budget. Page by page also means a directory of
            // tens of thousands of files is never held in memory whole, and a
            // stop between pages keeps everything queued so far.
            self.uploads
                .enqueue_assets(&album_id, &missing)
                .await
                .map_err(|cause| SyncError::QueueFailed { cause })?;
            ctx.advance(1);
            cursor = page.next;
            if cursor.is_none() {
                break;
            }
        }
        Ok(JobOutcome::Done)
    }
}
