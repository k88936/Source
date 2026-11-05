//! Sync: pushing the files of the registered local albums into their albums.
//!
//! A local album is a device directory the user pointed at one album. A pass
//! walks every registered directory one level deep and offers every file the
//! album does not already hold to the upload queue. The queue does the moving,
//! with its own retries and its own session resume; a pass only decides what to
//! offer.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::media::{LocalAsset, LocalMediaPort};
use crate::remote::RemoteStorage;
use crate::store::{
    AlbumId, AlbumStore, AssetStore, FolderStore, LocalAlbumRecord, LocalStore, Run, RunKey,
    RunStore, UploadQueueStore, UploadTaskId,
};
use crate::task::{Retry, Runner};

use super::run::{RunSink, RunTracker};
use super::uploads::UploadsService;

/// How many files one page of a device directory lists.
const LOCAL_PAGE: u32 = 200;

/// How many local albums are walked at once.
const SYNC_CONCURRENCY: usize = 2;

/// Which pass this is. The store compares keys and never learns what one means.
const SYNC_RUN: RunKey = RunKey::new("sync");

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
    pub async fn get_sync_run(&self) -> Result<Option<Run>, AppError> {
        self.catalog.get_run(SYNC_RUN).await
    }

    /// Start one pass over every registered local album, and return immediately.
    ///
    /// A pass already running is handed back as it is. The count is fixed when
    /// the pass starts, from what the device reports for each directory; a file
    /// deleted mid-walk simply never gets counted, and the pass still ends.
    pub async fn start_sync(self: &Arc<Self>, sink: RunSink) -> Result<Run, AppError> {
        let albums = self.catalog.list_local_albums().await?;
        let mut total = 0u32;
        for album in &albums {
            total += self.local.count_assets(&album.uri).await?;
        }
        let (run, started) = self.catalog.begin_run(SYNC_RUN, total).await?;
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
            // Everything this pass queued, started once at the end: one batch
            // means one concurrency budget however many albums the pass walks.
            let offered = Arc::new(Mutex::new(Vec::new()));
            runner
                .run(albums, tracker.get_cancel(), |album, _| {
                    let batch = Arc::clone(&batch);
                    let tracker = tracker.clone();
                    let offered = Arc::clone(&offered);
                    async move { batch.sync_local_album(&album, &tracker, &offered).await }
                })
                .await;
            let queued = std::mem::take(&mut *offered.lock().unwrap());
            sync.uploads.start(queued);
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
    /// A file is handled when it is already in the album or already sitting in
    /// the queue; only a directory that cannot be walked fails.
    async fn sync_local_album(
        &self,
        album: &LocalAlbumRecord,
        tracker: &RunTracker<C>,
        offered: &Mutex<Vec<UploadTaskId>>,
    ) -> Result<(), AppError> {
        let Some(folder) = self.catalog.get_folder(&album.album_id).await? else {
            // The album was removed; every file of the directory counts as
            // failed, so the pass reports what could not be pushed.
            let count = self.local.count_assets(&album.uri).await?;
            return tracker.advance(0, count).await;
        };
        let names = self.catalog.list_album_asset_names(&folder.id).await?;
        // Matched by name, so a directory of tens of thousands of files does
        // not turn every page into a scan of everything the album holds.
        let held: HashSet<&str> = names.iter().map(String::as_str).collect();

        let mut cursor: Option<String> = None;
        loop {
            if tracker.get_cancel().is_cancelled() {
                break;
            }
            let page = self
                .local
                .list_assets(&album.uri, cursor.as_deref(), LOCAL_PAGE)
                .await?;
            let count = page.items.len() as u32;
            let missing: Vec<LocalAsset> = page
                .items
                .into_iter()
                .filter(|asset| !held.contains(asset.name.as_str()))
                .collect();
            // Each page is offered as it is read, so an album of tens of
            // thousands of files is never held in memory at once, and a stop
            // between pages keeps everything queued so far.
            let album_id = AlbumId(album.album_id.clone());
            match self.uploads.enqueue_assets(&album_id, &missing).await {
                Ok(tasks) => {
                    tracker.advance(count, 0).await?;
                    offered
                        .lock()
                        .unwrap()
                        .extend(tasks.into_iter().map(|task| task.id));
                }
                Err(_) => tracker.advance(0, count).await?,
            }
            cursor = page.next;
            if cursor.is_none() {
                break;
            }
        }
        Ok(())
    }
}
