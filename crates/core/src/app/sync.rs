//! Sync: pushing the files of the registered local albums into their albums.
//!
//! A local album is a device directory the user pointed at one album. A pass
//! walks every registered directory one level deep and offers every file the
//! album does not already hold to the upload queue. The queue does the moving,
//! with its own retries and its own session resume; a pass only decides what to
//! offer.

use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::media::LocalMediaPort;
use crate::remote::RemoteStorage;
use crate::store::{
    AlbumId, AlbumStore, AssetStore, FolderStore, LocalAlbumRecord, LocalStore, Run, RunKind,
    RunStore, UploadQueueStore,
};
use crate::task::{Retry, Runner};

use super::run::{RunSink, RunTracker};
use super::uploads::{EnqueueUploadsParams, UploadsService};

/// How many files one page of a device directory lists.
const LOCAL_PAGE: u32 = 200;

/// How many local albums are walked at once.
const SYNC_CONCURRENCY: usize = 2;

pub struct SyncService<R, C, M> {
    catalog: C,
    local: M,
    uploads: Arc<UploadsService<R, C, M>>,
    /// The token of the sync pass running now, so a stop can reach it.
    syncing: Mutex<Option<CancellationToken>>,
}

impl<R, C, M> SyncService<R, C, M> {
    pub fn new(catalog: C, local: M, uploads: Arc<UploadsService<R, C, M>>) -> Self {
        Self {
            catalog,
            local,
            uploads,
            syncing: Mutex::new(None),
        }
    }
}

impl<R, C, M> SyncService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AlbumStore
        + AssetStore
        + FolderStore
        + LocalStore
        + UploadQueueStore
        + RunStore
        + Clone
        + 'static,
    M: LocalMediaPort + 'static,
{
    /// The newest sync pass, whatever state it is in.
    pub async fn sync_run(&self) -> Result<Option<Run>, AppError> {
        self.catalog.run(RunKind::Sync).await
    }

    /// Start one pass over every registered local album, and return immediately.
    ///
    /// A pass already running is handed back as it is. The count is fixed when
    /// the pass starts, from what the device reports for each directory; a file
    /// deleted mid-walk simply never gets counted, and the pass still ends.
    pub async fn start_sync(self: &Arc<Self>, sink: RunSink) -> Result<Run, AppError> {
        let albums = self.catalog.local_albums().await?;
        let total = albums
            .iter()
            .try_fold(0u32, |sum, album| Ok(sum + self.local.count(&album.uri)?))?;
        let (run, started) = self.catalog.begin_run(RunKind::Sync, total).await?;
        if !started {
            return Ok(run);
        }
        let cancel = CancellationToken::new();
        *self.syncing.lock().unwrap() = Some(cancel.clone());
        let tracker = RunTracker::new(self.catalog.clone(), &run, sink, cancel);
        tracker.announce(&run);

        let sync = Arc::clone(self);
        tokio::spawn(async move {
            // Listing a directory fails as a whole or not at all, and the files
            // it offers carry their own retries in the upload queue.
            let runner = Runner::new(SYNC_CONCURRENCY, Retry::NONE);
            let batch = Arc::clone(&sync);
            runner
                .run(albums, tracker.cancel(), |album, _| {
                    let batch = Arc::clone(&batch);
                    let tracker = tracker.clone();
                    async move { batch.sync_local_album(&album, &tracker).await }
                })
                .await;
            let _ = tracker.close().await;
            *sync.syncing.lock().unwrap() = None;
        });
        Ok(run)
    }

    /// Stop the pass after the directories already being walked.
    pub fn stop_sync(&self) {
        if let Some(cancel) = self.syncing.lock().unwrap().as_ref() {
            cancel.cancel();
        }
    }

    /// One local album: offer every file the album does not hold yet.
    ///
    /// A file is handled when it is already in the album, already waiting in
    /// the queue, or queued by this pass; it fails only when it could not be
    /// offered to the queue at all.
    async fn sync_local_album(
        &self,
        album: &LocalAlbumRecord,
        tracker: &RunTracker<C>,
    ) -> Result<(), AppError> {
        let Some(folder) = self.catalog.folder(&album.album_id).await? else {
            // The album was removed; every file of the directory counts as
            // failed, so the pass reports what could not be pushed.
            let count = self.local.count(&album.uri)?;
            return tracker.advance(0, count).await;
        };
        let names = self.catalog.album_asset_names(&folder.id).await?;

        let mut seen = 0u32;
        let mut candidates = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            if tracker.cancel().is_cancelled() {
                return Ok(());
            }
            let page = self
                .local
                .assets(&album.uri, cursor.as_deref(), LOCAL_PAGE)?;
            for asset in page.items {
                seen += 1;
                if !names.iter().any(|name| name == &asset.name) {
                    candidates.push(asset.id);
                }
            }
            cursor = page.next;
            if cursor.is_none() {
                break;
            }
        }

        if candidates.is_empty() {
            return tracker.advance(seen, 0).await;
        }
        let failed = candidates.len() as u32;
        let queued = self
            .uploads
            .enqueue(EnqueueUploadsParams {
                local_ids: candidates,
                album_id: AlbumId(album.album_id.clone()),
            })
            .await;
        match queued {
            Ok(tasks) => {
                tracker.advance(seen, 0).await?;
                // Start them now instead of leaving the queue for a later
                // trigger: the user asked for a sync, not for a queue.
                self.uploads
                    .start(tasks.into_iter().map(|task| task.id.0).collect());
                Ok(())
            }
            Err(_) => tracker.advance(seen - failed, failed).await,
        }
    }
}
