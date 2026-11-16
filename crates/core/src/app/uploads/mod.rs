//! Upload queue: enqueue, run, cancel, retry and progress persistence.

mod dto;
mod view;

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use crate::app::JobKind;
use crate::app::jobs::{Ctx, JobId, JobOutcome, JobPriority, Jobs};
use source_subscription::{Change, SubscriptionManager};
use tokio::sync::mpsc;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::exif::ExifData;
use crate::mapping::asset_record;
use crate::media::{LocalAsset, LocalMediaPort, MediaError};
use crate::remote::storage::join_uri;
use crate::remote::{
    MediaFetchError, RemoteEntry, RemoteError, RemoteStorage, UploadProgress, UploadRequest,
    UploadUriError,
};
use crate::store::{
    AlbumId, AssetStore, CatalogError, FolderId, FolderStore, LocalStore,
    UploadQueueStore, UploadState, UploadTask, UploadTaskId,
};
use crate::task::{Attempt, Retry, Step, Try, attempt};
use crate::time::now_ms;

use super::PROGRESS_WRITE_INTERVAL;

pub use dto::*;
pub use view::*;

/// What the upload queue can fail with.
///
/// The queue is the only layer that decides whether a failure is worth another
/// attempt, so the server-side causes arrive typed: a chunk the connection
/// dropped can be resumed, a file that left the device cannot.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum UploadError {
    /// The index could not answer.
    Catalog { cause: CatalogError },
    /// The album the batch targets is gone.
    AlbumMissing { album: String },
    /// The queue row the caller points at is gone.
    TaskMissing { task: String },
    /// The row is not in a state that can be re-queued.
    TaskNotFailed { task: String },
    /// The local file behind the row left the device.
    LocalFileMissing { asset: String },
    /// The device library failed.
    LocalMedia { cause: MediaError },
    /// The upload attempt itself failed.
    Upload { task: String, cause: UploadUriError },
    /// Reading the server copy that a taken name points at failed.
    Remote { cause: MediaFetchError },
}

impl UploadError {
    fn catalog(cause: CatalogError) -> Self {
        UploadError::Catalog { cause }
    }

    /// Whether a further attempt could still succeed.
    ///
    /// A body that could not be read, or a name the server refused, stays
    /// refused; a dropped connection or a refused chunk may not.
    fn is_transient(&self) -> bool {
        matches!(
            self,
            UploadError::Upload {
                cause: UploadUriError::ChunkFailed { .. } | UploadUriError::Unreachable { .. },
                ..
            }
        )
    }

    /// Whether the attempt stopped because the caller asked it to.
    fn is_interrupted(&self) -> bool {
        matches!(
            self,
            UploadError::Upload {
                cause: UploadUriError::Interrupted,
                ..
            }
        )
    }

    /// The bytes kept in the queue row, so a later listing can read the failure
    /// back into this type.
    fn stored(&self) -> Result<String, UploadError> {
        serde_json::to_string(self).map_err(|cause| UploadError::Catalog {
            cause: CatalogError::EncodeFailed {
                cause: cause.to_string(),
            },
        })
    }
}

/// Result of one upload attempt.
#[derive(Debug, Clone)]
pub struct UploadOutcome {
    pub task: UploadTask,
}

/// Automatic retries after the first failure; the first wait is 3s, then it
/// doubles.
const UPLOAD_RETRY: Retry = Retry {
    attempts: 3,
    base: std::time::Duration::from_secs(3),
};

/// How many uploads the board holds before the backlog makes it grow.
///
/// The rest wait as rows in the table, so the board - and every broadcast of it
/// - never carries the whole backlog, whatever the size of a first sync.
const MIN_UPLOAD_IN_FLIGHT: usize = 32;

/// The most the board ever reserves, however large the backlog gets.
const MAX_UPLOAD_IN_FLIGHT: usize = 128;

/// How often the queue looks at the backlog for rows to start.
///
/// Short enough that a file a user just picked starts without a visible wait,
/// long enough that the board is still full of uploads when the next tick
/// comes, so a tick never runs dry while there is work.
const UPLOAD_TICK: Duration = Duration::from_millis(250);

/// How many bytes of each window are compared when a name is already taken.
const CONFLICT_SAMPLE: i64 = 64 * 1024;

/// The windows compared when the server already holds the name: head, middle
/// and tail. A file no larger than one window is compared whole.
fn sample_windows(size: i64) -> Vec<(i64, u32)> {
    if size <= 0 {
        return Vec::new();
    }
    if size <= CONFLICT_SAMPLE {
        return vec![(0, size as u32)];
    }
    vec![
        (0, CONFLICT_SAMPLE as u32),
        ((size - CONFLICT_SAMPLE) / 2, CONFLICT_SAMPLE as u32),
        (size - CONFLICT_SAMPLE, CONFLICT_SAMPLE as u32),
    ]
}

/// Keeps two tasks enqueued in the same millisecond apart, across every batch.
static TASK_SEQ: AtomicU64 = AtomicU64::new(0);

/// Where an upload attempt reports to: progress and state changes. Built by the
/// composition root, so the queue never depends on a window.
#[derive(Clone)]
pub struct UploadSinks {
    pub state: Arc<dyn Fn(&UploadStateEvent) + Send + Sync>,
}

pub struct UploadsService<R, C, M>
where
    C: UploadQueueStore + 'static,
{
    cloud: R,
    catalog: C,
    local: M,
    sinks: UploadSinks,
    jobs: Jobs,
    /// Every page's subscription to the rows whose bytes are moving.
    running: Arc<SubscriptionManager<RunningUploads<C>>>,
    /// How many uploads the board may hold, before this tick moves it.
    target: AtomicUsize,
}

impl<R, C, M> UploadsService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AssetStore + FolderStore + LocalStore + UploadQueueStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    pub fn new(cloud: R, catalog: C, local: M, sinks: UploadSinks, jobs: Jobs) -> Arc<Self> {
        let running = Arc::new(SubscriptionManager::new(Arc::new(RunningUploads::new(
            catalog.clone(),
        ))));
        let uploads = Arc::new(Self {
            cloud,
            catalog,
            local,
            sinks,
            jobs,
            running,
            target: AtomicUsize::new(MIN_UPLOAD_IN_FLIGHT),
        });
        // The queue keeps itself fed from birth, so recovering rows is all a
        // startup has to do.
        uploads.watch();
        uploads
    }
}

impl<R, C, M> UploadsService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AssetStore + FolderStore + LocalStore + UploadQueueStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    /// Offer the files a user picked. They are read from the device, then
    /// queued exactly like a sync pass queues what it finds.
    pub async fn enqueue_files(
        self: &Arc<Self>,
        params: EnqueueUploadsParams,
    ) -> Result<Vec<QueuedUpload>, UploadError> {
        self.enqueue_files_impl(params).await
    }

    /// What the queue is doing now, for the card that shows it.
    pub async fn list_tasks(&self) -> Result<Vec<QueuedUpload>, UploadError> {
        self.list_tasks_impl().await
    }

    /// The page's view of the rows whose bytes are moving.
    pub fn running(&self) -> Arc<SubscriptionManager<RunningUploads<C>>> {
        Arc::clone(&self.running)
    }

    /// One commit, answered for every page subscribed to the moving rows.
    pub async fn apply_changes(&self, changes: &[Change]) {
        self.running.apply(changes).await;
    }

    pub async fn get_task(&self, id: &UploadTaskId) -> Result<Option<QueuedUpload>, UploadError> {
        self.get_task_impl(id).await
    }

    /// Cancel one task, whether or not it is running.
    ///
    /// The cancelled state is written here and is what stops a running attempt
    /// between two chunks; the attempt keeps its session, so retrying the task
    /// later resumes instead of starting over.
    pub async fn cancel(&self, task_id: &UploadTaskId) -> Result<(), UploadError> {
        self.cancel_impl(task_id).await
    }

    pub async fn retry(
        self: &Arc<Self>,
        task_id: &UploadTaskId,
    ) -> Result<QueuedUpload, UploadError> {
        self.retry_impl(task_id).await
    }

    /// Put back what a previous process left in flight and queue it again.
    ///
    /// Called once at startup: in-flight rows go back to queued, so the next
    /// attempt loads their durable session and resumes instead of starting over.
    pub async fn recover(self: &Arc<Self>) -> Result<Vec<QueuedUpload>, UploadError> {
        self.recover_impl().await
    }
}

// ---------------------------------------------------------------------
// Use case implementations
// ---------------------------------------------------------------------

impl<R, C, M> UploadsService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AssetStore + FolderStore + LocalStore + UploadQueueStore + Clone + 'static,
    M: LocalMediaPort + 'static,
{
    async fn enqueue_files_impl(
        self: &Arc<Self>,
        params: EnqueueUploadsParams,
    ) -> Result<Vec<QueuedUpload>, UploadError> {
        let mut assets = Vec::with_capacity(params.local_ids.len());
        for local_id in params.local_ids {
            if let Some(asset) = self
                .local
                .get_asset(local_id.as_str())
                .await
                .map_err(|cause| UploadError::LocalMedia { cause })?
            {
                assets.push(asset);
            }
        }
        // A file the user picked has nothing else staging it, so the row that
        // the upload reads from is written here.
        self.catalog
            .save_local_assets(None, &assets)
            .await
            .map_err(UploadError::catalog)?;
        let tasks = self.enqueue_assets(&params.album_id, None, &assets).await?;
        // A file the user picked starts now, pause or not: the queue ranks it
        // above the backlog a scan leaves behind.
        for task in &tasks {
            self.submit(task.clone(), JobPriority::RealTime);
        }
        Ok(tasks.into_iter().map(QueuedUpload::from).collect())
    }

    /// Queue files the caller already holds and has already staged.
    ///
    /// One file has one row per destination folder: a file already waiting or
    /// in flight is not queued twice, and a row that ended is reused instead of
    /// copied, so a scan pass after pass never grows the queue.
    ///
    /// `source` is the local album row that asked for the batch, so the queue
    /// can be read back per mapping; a batch the user hand-picked has none.
    pub(super) async fn enqueue_assets(
        &self,
        album_id: &AlbumId,
        source: Option<&str>,
        assets: &[LocalAsset],
    ) -> Result<Vec<UploadTask>, UploadError> {
        if assets.is_empty() {
            return Ok(Vec::new());
        }
        let dest = self
            .catalog
            .get_folder(&album_id.0)
            .await
            .map_err(UploadError::catalog)?
            .ok_or_else(|| UploadError::AlbumMissing {
                album: album_id.0.clone(),
            })?;
        let mut tasks = Vec::new();
        for asset in assets {
            let task = UploadTask {
                id: UploadTaskId(format!(
                    "up-{}-{}",
                    now_ms(),
                    TASK_SEQ.fetch_add(1, Ordering::Relaxed)
                )),
                local_asset_id: asset.id.clone(),
                local_album_id: source.map(str::to_owned),
                dest_folder_id: FolderId(dest.id.clone()),
                name: asset.name.clone(),
                dest_uri: dest.uri.clone(),
                dest_name: Some(dest.name.clone()),
                state: UploadState::Queued,
                uploaded: 0,
                total: asset.size.max(0),
                speed: 0,
                eta_seconds: None,
                error: None,
            };
            if let Some(task) = self
                .catalog
                .enqueue_upload(&task)
                .await
                .map_err(UploadError::catalog)?
            {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    /// Start the oldest rows the board has room for.
    ///
    /// The table is the backlog, and this is the only place a row becomes work.
    /// Submitting the same row twice is harmless, since one id is one job, so a
    /// tick racing another never runs a row twice.
    ///
    /// The room moves like a vector's capacity: it doubles while work really
    /// waits outside the board, and halves while what is left cannot fill a
    /// quarter of it. A first sync of thousands of files therefore reserves more
    /// than the small steady state, and the board comes back down once the
    /// import is done instead of staying large for the rest of the process.
    async fn spawn_pending(self: &Arc<Self>) -> Result<(), UploadError> {
        // The queue ticks from the first moment of the process, often before the
        // stored session has been checked. Starting a row then fails it as an
        // expired session, and the backlog loses its place for a reason that
        // belongs to the start rather than to the file.
        if !self.cloud.is_live() {
            return Ok(());
        }
        let board = self.jobs.list();
        let live = board
            .iter()
            .filter(|job| job.kind == JobKind::Upload && job.state.is_live())
            .count();
        // Naming the uploads already on the board is what separates "work is
        // left on the table" from "the board is merely full": a submitted row
        // stays queued in the table until its attempt writes CreatingSession.
        let on_board: HashSet<&str> = board
            .iter()
            .filter(|job| job.kind == JobKind::Upload)
            .map(|job| job.id.0.as_str())
            .collect();
        let target = self.target.load(Ordering::Relaxed);
        let free = target.saturating_sub(live);
        // One row past the whole room: the board holds at most `target`
        // uploads, so a row beyond that page is a row nobody has picked up.
        let queued: Vec<UploadTask> = self
            .catalog
            .next_queued((target + 1) as i64)
            .await
            .map_err(UploadError::catalog)?
            .into_iter()
            .filter(|task| !on_board.contains(task.id.0.as_str()))
            .collect();
        if !queued.is_empty() {
            // Rows nobody has picked up are still waiting: grow.
            if target < MAX_UPLOAD_IN_FLIGHT {
                self.target
                    .store((target * 2).min(MAX_UPLOAD_IN_FLIGHT), Ordering::Relaxed);
            }
        } else if live * 4 < target && target > MIN_UPLOAD_IN_FLIGHT {
            // Every waiting row is already on the board, and the board is not
            // even a quarter full: the import is over, so shrink. A quarter, not
            // a half, leaves a dead band between the two thresholds, so one more
            // file cannot grow it straight back.
            self.target
                .store((target / 2).max(MIN_UPLOAD_IN_FLIGHT), Ordering::Relaxed);
        }
        for task in queued.into_iter().take(free) {
            self.submit(task, JobPriority::Normal(0));
        }
        Ok(())
    }

    /// Keep the board filled, off the job budget.
    ///
    /// A timer is not a job, so the board only ever holds uploads, and a tick
    /// never takes a slot from the work it starts. Started once, at
    /// construction, so nothing has to remember to start it later.
    fn watch(self: &Arc<Self>) {
        let uploads = Arc::clone(self);
        self.jobs.timer(UPLOAD_TICK, move || {
            let uploads = Arc::clone(&uploads);
            async move { uploads.spawn_pending().await }
        });
    }

    /// Queue one task as a job, so the queue decides when this file moves.
    ///
    /// A row that is already queued or uploading is one job already: pressing
    /// retry twice, or a scan queueing the same file again, never puts two
    /// attempts on the same bytes.
    fn submit(self: &Arc<Self>, task: UploadTask, priority: JobPriority) {
        let uploads = Arc::clone(self);
        let task_id = task.id;
        self.jobs.submit(
            JobKind::Upload,
            JobId::from(task_id.0.clone()),
            priority,
            move |ctx| {
                let uploads = Arc::clone(&uploads);
                let task_id = task_id.clone();
                async move {
                    let ran = ctx
                        .run_paused(|| {
                            attempt(UPLOAD_RETRY, ctx.cancel(), |try_number| {
                                uploads.try_upload(&task_id, &ctx, try_number)
                            })
                        })
                        .await;
                    match ran.value {
                        Attempt::Done(_) => Ok(JobOutcome::Done),
                        Attempt::Stopped => {
                            uploads.park_upload(&task_id).await?;
                            Ok(if ran.paused {
                                JobOutcome::Paused
                            } else {
                                JobOutcome::Done
                            })
                        }
                        Attempt::Failed(error) => Err(error),
                    }
                }
            },
        );
    }

    async fn list_tasks_impl(&self) -> Result<Vec<QueuedUpload>, UploadError> {
        let rows = self
            .catalog
            .list_upload_tasks()
            .await
            .map_err(UploadError::catalog)?;
        Ok(rows.into_iter().map(QueuedUpload::from).collect())
    }

    async fn get_task_impl(&self, id: &UploadTaskId) -> Result<Option<QueuedUpload>, UploadError> {
        let row = self
            .catalog
            .get_upload_task(&id.0)
            .await
            .map_err(UploadError::catalog)?;
        Ok(row.map(QueuedUpload::from))
    }

    async fn cancel_impl(&self, task_id: &UploadTaskId) -> Result<(), UploadError> {
        let Some(mut task) = self
            .catalog
            .get_upload_task(&task_id.0)
            .await
            .map_err(UploadError::catalog)?
        else {
            return Err(UploadError::TaskMissing {
                task: task_id.0.clone(),
            });
        };
        if matches!(task.state, UploadState::Done | UploadState::Failed) {
            return Ok(());
        }
        self.set_state(&mut task, UploadState::Cancelled).await?;
        self.jobs
            .cancel_item(JobKind::Upload, &JobId::from(task_id.0.clone()));
        Ok(())
    }

    async fn retry_impl(
        self: &Arc<Self>,
        task_id: &UploadTaskId,
    ) -> Result<QueuedUpload, UploadError> {
        let Some(mut task) = self
            .catalog
            .get_upload_task(&task_id.0)
            .await
            .map_err(UploadError::catalog)?
        else {
            return Err(UploadError::TaskMissing {
                task: task_id.0.clone(),
            });
        };
        task.state = UploadState::Queued;
        task.error = None;
        task.uploaded = 0;
        task.speed = 0;
        task.eta_seconds = None;
        self.catalog
            .update_upload(&task)
            .await
            .map_err(UploadError::catalog)?;
        // A retry is the user picking the file again, so it runs like a picked
        // upload instead of waiting for the next tick.
        self.submit(task.clone(), JobPriority::RealTime);
        Ok(QueuedUpload::from(task))
    }

    /// Re-queue a failed task for an automatic retry.
    ///
    /// Unlike [`Self::retry`] this keeps the progress counters: the uploader
    /// resumes from its session, and the UI should not flash back to 0% while an
    /// unattended retry is waiting.
    ///
    /// Only a failed task can be re-queued: the retry loop calls this after a
    /// backoff, and reviving a task the user cancelled meanwhile would undo the
    /// cancellation.
    async fn requeue(&self, task_id: &UploadTaskId) -> Result<UploadTask, UploadError> {
        let Some(mut task) = self
            .catalog
            .get_upload_task(&task_id.0)
            .await
            .map_err(UploadError::catalog)?
        else {
            return Err(UploadError::TaskMissing {
                task: task_id.0.clone(),
            });
        };
        if task.state != UploadState::Failed {
            return Err(UploadError::TaskNotFailed {
                task: task_id.0.clone(),
            });
        }
        task.state = UploadState::Queued;
        task.error = None;
        task.speed = 0;
        task.eta_seconds = None;
        self.catalog
            .update_upload(&task)
            .await
            .map_err(UploadError::catalog)?;
        Ok(task)
    }

    /// One attempt, including the states it announces before and after.
    ///
    /// It reports its own verdict: stopping and "worth another visit" are
    /// decisions of the work, not of the loop that performs the wait.
    async fn try_upload(
        &self,
        id: &UploadTaskId,
        ctx: &Ctx<JobKind>,
        attempt: Try,
    ) -> Step<(), UploadError> {
        match self.try_upload_once(id, ctx, attempt).await {
            Ok(()) => Step::Done(()),
            Err(error) if error.is_interrupted() => Step::Stopped,
            Err(error) if error.is_transient() => Step::Retry(error),
            Err(error) => Step::Failed(error),
        }
    }

    async fn try_upload_once(
        &self,
        id: &UploadTaskId,
        ctx: &Ctx<JobKind>,
        attempt: Try,
    ) -> Result<(), UploadError> {
        if let Some(task) = self
            .catalog
            .get_upload_task(&id.0)
            .await
            .map_err(UploadError::catalog)?
        {
            (self.sinks.state)(&UploadStateEvent {
                task_id: task.id,
                state: UploadState::Queued,
                error: None,
            });
        }
        match self.run(id, ctx).await {
            Ok(outcome) => {
                (self.sinks.state)(&UploadStateEvent {
                    task_id: outcome.task.id.clone(),
                    state: outcome.task.state,
                    error: None,
                });
                Ok(())
            }
            Err(error) => {
                // A stop is not a failure: the job wrapper parks a paused row
                // back to queued, and a user cancel already wrote Cancelled.
                if error.is_interrupted() {
                    return Err(error);
                }
                // Transient failures resume from the session on the next try;
                // anything else (auth, permission, conflict) is final.
                if error.is_transient() && attempt.number < attempt.retry.attempts {
                    // Re-queue first: reviving a task the user cancelled while
                    // the attempt ran fails here and ends the retries.
                    self.requeue(id).await?;
                    return Err(error);
                }
                // Persisted before it is announced: a row left queued would be
                // resumed at every startup, so a file that is gone from the
                // device would fail again and again, forever.
                let task = self.fail(id, error).await?;
                (self.sinks.state)(&UploadStateEvent {
                    task_id: task.id,
                    state: task.state,
                    error: task.error,
                });
                Ok(())
            }
        }
    }

    /// Put a paused attempt back in the table, without counting a retry.
    ///
    /// The upload session is left alone, so resume continues from its chunks.
    async fn park_upload(&self, id: &UploadTaskId) -> Result<(), UploadError> {
        let Some(mut task) = self
            .catalog
            .get_upload_task(&id.0)
            .await
            .map_err(UploadError::catalog)?
        else {
            return Ok(());
        };
        if task.state == UploadState::Cancelled {
            return Ok(());
        }
        task.state = UploadState::Queued;
        task.error = None;
        task.speed = 0;
        task.eta_seconds = None;
        self.catalog
            .update_upload(&task)
            .await
            .map_err(UploadError::catalog)?;
        (self.sinks.state)(&UploadStateEvent {
            task_id: task.id,
            state: UploadState::Queued,
            error: None,
        });
        Ok(())
    }

    /// Write the terminal failure, unless the task was cancelled meanwhile.
    async fn fail(
        &self,
        id: &UploadTaskId,
        error: UploadError,
    ) -> Result<QueuedUpload, UploadError> {
        let Some(mut task) = self
            .catalog
            .get_upload_task(&id.0)
            .await
            .map_err(UploadError::catalog)?
        else {
            return Err(error);
        };
        if task.state == UploadState::Cancelled {
            return Ok(QueuedUpload::from(task));
        }
        task.state = UploadState::Failed;
        task.error = Some(error.stored()?);
        task.speed = 0;
        task.eta_seconds = None;
        self.catalog
            .update_upload(&task)
            .await
            .map_err(UploadError::catalog)?;
        Ok(QueuedUpload::from(task))
    }

    /// Execute one queued task, reporting its bytes to its job.
    async fn run(
        &self,
        task_id: &UploadTaskId,
        ctx: &Ctx<JobKind>,
    ) -> Result<UploadOutcome, UploadError> {
        let mut task = self
            .catalog
            .get_upload_task(&task_id.0)
            .await
            .map_err(UploadError::catalog)?
            .ok_or_else(|| UploadError::TaskMissing {
                task: task_id.0.clone(),
            })?;
        if task.state != UploadState::Queued {
            return Ok(UploadOutcome { task });
        }
        let local = match self
            .catalog
            .get_local_asset(task.local_asset_id.as_str())
            .await
            .map_err(UploadError::catalog)?
        {
            Some(asset) => Some(asset),
            None => self
                .local
                .get_asset(task.local_asset_id.as_str())
                .await
                .map_err(|cause| UploadError::LocalMedia { cause })?,
        }
        .ok_or_else(|| UploadError::LocalFileMissing {
            asset: task.local_asset_id.0.clone(),
        })?;

        let max_total = task.total;
        let session_key = task.id.0.clone();
        // A cancel that lands while this attempt is being set up must not be
        // lost: the job's own token is already live. The uploader stops between
        // two chunks and keeps its session, so a later retry resumes instead of
        // starting over.
        let cancel = ctx.cancel().clone();
        if max_total > 0 {
            ctx.total(max_total as u64);
        }
        ctx.set_done(task.uploaded.max(0) as u64);

        self.set_state(&mut task, UploadState::CreatingSession)
            .await?;
        self.set_state(&mut task, UploadState::Uploading).await?;

        // Single writer: the progress callback only hands the newest sample to a
        // capacity-1 channel (stale samples are dropped) and the writer persists
        // at most one sample per `PROGRESS_WRITE_INTERVAL`.
        let (progress_tx, mut progress_rx) = mpsc::channel::<UploadProgress>(1);
        let writer = ctx.spawn({
            let catalog = self.catalog.clone();
            let task_id = task.id.0.clone();
            async move {
                while let Some(update) = progress_rx.recv().await {
                    let mut latest = update;
                    while let Ok(next) = progress_rx.try_recv() {
                        latest = next;
                    }
                    if let Ok(Some(mut current)) = catalog.get_upload_task(&task_id).await {
                        current.uploaded = latest.uploaded;
                        current.total = if latest.total > 0 {
                            latest.total
                        } else {
                            max_total
                        };
                        current.speed = latest.speed;
                        current.eta_seconds = latest.eta_seconds;
                        let _ = catalog.update_upload(&current).await;
                    }
                    tokio::time::sleep(PROGRESS_WRITE_INTERVAL).await;
                }
            }
        });
        let progress = Arc::new({
            let ctx = ctx.clone();
            let reported = AtomicU64::new(task.uploaded.max(0) as u64);
            move |p: UploadProgress| {
                let _ = progress_tx.try_send(p);
                // One chunk is one step. The job counts bytes, so a big video
                // moves its row in the queue instead of sitting at 0% until it
                // ends.
                let sent = p.uploaded.max(0) as u64;
                let previous = reported.swap(sent, Ordering::Relaxed);
                if sent > previous {
                    ctx.advance(sent - previous);
                }
            }
        });

        let result = self
            .cloud
            .upload_uri(UploadRequest {
                task_id: task.id.clone(),
                source_uri: local.uri.clone(),
                size: local.size,
                dest_uri: task.dest_uri.clone(),
                name: local.name.clone(),
                mime: local.mime.clone(),
                cancel: cancel.clone(),
                progress,
            })
            .await
            .map_err(|cause| UploadError::Upload {
                task: session_key.clone(),
                cause,
            });

        // The terminal state written below is authoritative: stop the throttled
        // writer first so it cannot overwrite it with a stale sample.
        writer.abort();

        // The attempt may have been cancelled while it ran: its result must not
        // turn a cancelled task back into a finished or failed one.
        if let Some(current) = self
            .catalog
            .get_upload_task(&session_key)
            .await
            .map_err(UploadError::catalog)?
            && current.state == UploadState::Cancelled
        {
            return Ok(UploadOutcome {
                task: current,
            });
        }

        let entry = match result {
            Ok(entry) => entry,
            // A taken name is the server saying the folder already has a file
            // called this. It is only a failure when the bytes differ: the same
            // size and the same sampled windows mean the server already holds
            // this file, so the row finishes without uploading anything.
            Err(
                err @ UploadError::Upload {
                    cause: UploadUriError::Conflict { .. },
                    ..
                },
            ) => {
                let remote_uri = join_uri(&task.dest_uri, &local.name);
                match self.same_remote_file(&local, &remote_uri).await? {
                    Some(entry) => entry,
                    None => return Err(err),
                }
            }
            Err(err) => return Err(err),
        };
        self.set_state(&mut task, UploadState::Completing).await?;
        let client_exif = (local.date_taken > 0).then_some(ExifData {
            taken_at: Some(local.date_taken),
            width: local.width,
            height: local.height,
        });
        let record = asset_record(&entry, &task.dest_folder_id, client_exif.as_ref());
        self.catalog
            .upsert_asset(&record)
            .await
            .map_err(UploadError::catalog)?;
        task.state = UploadState::Done;
        task.uploaded = task.total;
        task.speed = 0;
        task.eta_seconds = None;
        task.error = None;
        self.catalog
            .update_upload(&task)
            .await
            .map_err(UploadError::catalog)?;
        Ok(UploadOutcome { task })
    }

    /// Decide an upload the server refused because its name is taken.
    ///
    /// The row is a duplicate only when the server already holds this exact
    /// file: the size settles most cases for free, then three 64 KB windows of
    /// the remote copy are compared with the same offsets on the device. The
    /// remote entry comes back so the success path can index it as if this
    /// attempt had uploaded it.
    async fn same_remote_file(
        &self,
        local: &LocalAsset,
        remote_uri: &str,
    ) -> Result<Option<RemoteEntry>, UploadError> {
        let remote = self
            .cloud
            .get_file_info(remote_uri)
            .await
            .map_err(|cause| UploadError::Remote { cause })?;
        if remote.size != local.size {
            return Ok(None);
        }
        for (offset, len) in sample_windows(local.size) {
            let range = format!("bytes={offset}-{}", offset + i64::from(len) - 1);
            let answer = self
                .cloud
                .get_media_range(remote_uri, Some(&range))
                .await
                .map_err(|cause| UploadError::Remote { cause })?;
            // A range request answers with a status, so a refused read looks
            // like a body of bytes. It is not an answer about the content.
            if !answer.is_success() {
                return Err(UploadError::Remote {
                    cause: MediaFetchError::Transport {
                        cause: RemoteError::Rejected {
                            status: i32::from(answer.status),
                            message: "范围读取失败".to_string(),
                        },
                    },
                });
            }
            let mine = self
                .local
                .read_range(&local.uri, offset as u64, len)
                .await
                .map_err(|cause| UploadError::LocalMedia { cause })?;
            if answer.body != mine {
                return Ok(None);
            }
        }
        Ok(Some(remote))
    }

    async fn set_state(
        &self,
        task: &mut UploadTask,
        state: UploadState,
    ) -> Result<(), UploadError> {
        task.state = state;
        self.catalog
            .update_upload(task)
            .await
            .map_err(UploadError::catalog)
    }

    async fn recover_impl(&self) -> Result<Vec<QueuedUpload>, UploadError> {
        let rows = self
            .catalog
            .recover_uploads()
            .await
            .map_err(UploadError::catalog)?;
        Ok(rows.into_iter().map(QueuedUpload::from).collect())
    }
}
