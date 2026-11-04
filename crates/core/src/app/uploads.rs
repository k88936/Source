//! Upload queue: enqueue, run, cancel, retry and progress persistence.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::error::{AppError, ErrorCode};
use crate::exif::ExifData;
use crate::mapping::asset_record;
use crate::media::LocalMediaPort;
use crate::remote::{RemoteStorage, UploadProgress, UploadRequest};
use crate::store::{
    AlbumId, AssetStore, AssetSummary, FolderId, FolderStore, LocalStore, UploadQueueStore,
    UploadState, UploadTask, UploadTaskId,
};
use crate::time::now_ms;

use super::PROGRESS_WRITE_INTERVAL;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EnqueueUploadsParams {
    pub local_ids: Vec<String>,
    /// The album (remote folder) every file of this batch lands in.
    pub album_id: AlbumId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadTaskParams {
    pub task_id: UploadTaskId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadProgressEvent {
    pub task_id: UploadTaskId,
    pub uploaded: i64,
    pub total: i64,
    pub speed: i64,
    pub eta_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadStateEvent {
    pub task_id: UploadTaskId,
    pub state: UploadState,
    pub error: Option<AppError>,
}

/// Result of one upload attempt.
///
/// `asset` carries the catalog row written on success, so the UI can insert the
/// new tile straight from the completion event instead of reloading a page.
#[derive(Debug, Clone)]
pub struct UploadOutcome {
    pub task: UploadTask,
    pub asset: Option<AssetSummary>,
}

pub struct UploadsService<R, C, M> {
    cloud: R,
    catalog: C,
    local: M,
    /// Cancellation flags of the upload attempts currently running.
    cancels: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl<R, C, M> UploadsService<R, C, M> {
    pub fn new(cloud: R, catalog: C, local: M) -> Self {
        Self {
            cloud,
            catalog,
            local,
            cancels: Mutex::new(HashMap::new()),
        }
    }
}

impl<R, C, M> UploadsService<R, C, M>
where
    R: RemoteStorage,
    C: AssetStore + FolderStore + LocalStore + UploadQueueStore + Clone + 'static,
    M: LocalMediaPort,
{
    pub async fn enqueue(&self, params: EnqueueUploadsParams) -> Result<Vec<UploadTask>, AppError> {
        if params.local_ids.is_empty() {
            return Ok(Vec::new());
        }
        let dest = self
            .catalog
            .folder(&params.album_id.0)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let mut tasks = Vec::new();
        for local_id in params.local_ids {
            let Some(asset) = self.local.asset(&local_id)? else {
                continue;
            };
            self.catalog.save_local_asset(&asset).await?;
            let task = UploadTask {
                id: UploadTaskId(format!("up-{}-{}", now_ms(), tasks.len())),
                local_asset_id: asset.id.clone(),
                name: asset.name.clone(),
                dest_uri: dest.uri.clone(),
                dest_name: Some(dest.name.clone()),
                state: UploadState::Queued,
                uploaded: 0,
                total: asset.size.max(0),
                speed: 0,
                eta_seconds: None,
                attempts: 0,
                error: None,
            };
            self.catalog.enqueue_upload(&task).await?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub async fn tasks(&self) -> Result<Vec<UploadTask>, AppError> {
        self.catalog.upload_tasks().await
    }

    pub async fn task(&self, id: &str) -> Result<Option<UploadTask>, AppError> {
        self.catalog.upload_task(id).await
    }

    pub async fn cancel(&self, task_id: &str) -> Result<(), AppError> {
        let Some(mut task) = self.catalog.upload_task(task_id).await? else {
            return Err(AppError::not_found("上传任务不存在"));
        };
        if matches!(task.state, UploadState::Done | UploadState::Failed) {
            return Ok(());
        }
        task.state = UploadState::Cancelled;
        self.catalog.update_upload(&task).await?;
        // A running attempt stops between two chunks and keeps its session, so
        // retrying the task later resumes instead of starting over.
        if let Some(cancel) = self.cancels.lock().unwrap().get(task_id) {
            cancel.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    pub async fn retry(&self, task_id: &str) -> Result<UploadTask, AppError> {
        let Some(mut task) = self.catalog.upload_task(task_id).await? else {
            return Err(AppError::not_found("上传任务不存在"));
        };
        task.state = UploadState::Queued;
        task.error = None;
        task.uploaded = 0;
        task.speed = 0;
        task.eta_seconds = None;
        task.attempts += 1;
        self.catalog.update_upload(&task).await?;
        Ok(task)
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
    pub async fn requeue(&self, task_id: &str) -> Result<UploadTask, AppError> {
        let Some(mut task) = self.catalog.upload_task(task_id).await? else {
            return Err(AppError::not_found("上传任务不存在"));
        };
        if task.state != UploadState::Failed {
            return Err(AppError::new(
                ErrorCode::Conflict,
                "上传任务已被取消或已完成".to_string(),
                false,
            ));
        }
        task.state = UploadState::Queued;
        task.error = None;
        task.speed = 0;
        task.eta_seconds = None;
        task.attempts += 1;
        self.catalog.update_upload(&task).await?;
        Ok(task)
    }

    /// Execute one queued task. The caller (Tauri runtime) is responsible for
    /// scheduling and for forwarding `on_progress` to the webview.
    pub async fn run(
        &self,
        task_id: &str,
        on_progress: Arc<dyn Fn(UploadProgressEvent) + Send + Sync>,
    ) -> Result<UploadOutcome, AppError> {
        let mut task = self
            .catalog
            .upload_task(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("上传任务不存在"))?;
        if task.state != UploadState::Queued {
            return Ok(UploadOutcome { task, asset: None });
        }
        let local = self
            .catalog
            .local_asset(&task.local_asset_id)
            .await?
            .or(self.local.asset(&task.local_asset_id)?)
            .ok_or_else(|| AppError::not_found("本机文件不存在"))?;

        let max_total = task.total;
        let display_task_id = task.id.0.clone();
        let session_key = display_task_id.clone();
        // Registered before the first state write: a cancellation that lands
        // while this attempt is being set up must not be lost, and must not be
        // overwritten by the states written below.
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancels
            .lock()
            .unwrap()
            .insert(session_key.clone(), Arc::clone(&cancel));
        if cancel.load(Ordering::SeqCst) {
            self.cancels.lock().unwrap().remove(&session_key);
            task.state = UploadState::Cancelled;
            return Ok(UploadOutcome { task, asset: None });
        }

        self.set_state(&mut task, UploadState::CreatingSession)
            .await?;
        let folder_id = self
            .catalog
            .folder_by_uri(&task.dest_uri)
            .await?
            .map(|f| f.id)
            .unwrap_or_else(|| task.dest_uri.clone());
        self.set_state(&mut task, UploadState::Uploading).await?;

        // Single writer: the progress callback only hands the newest sample to a
        // capacity-1 channel (stale samples are dropped) and the writer persists
        // at most one sample per `PROGRESS_WRITE_INTERVAL`.
        let (progress_tx, mut progress_rx) = mpsc::channel::<UploadProgress>(1);
        let writer = tokio::spawn({
            let catalog = self.catalog.clone();
            let task_id = display_task_id.clone();
            async move {
                while let Some(update) = progress_rx.recv().await {
                    let mut latest = update;
                    while let Ok(next) = progress_rx.try_recv() {
                        latest = next;
                    }
                    if let Ok(Some(mut current)) = catalog.upload_task(&task_id).await {
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
        let progress = Arc::new(move |p: UploadProgress| {
            let _ = progress_tx.try_send(p);
            on_progress(UploadProgressEvent {
                task_id: UploadTaskId(display_task_id.clone()),
                uploaded: p.uploaded,
                total: if p.total > 0 { p.total } else { max_total },
                speed: p.speed,
                eta_seconds: p.eta_seconds,
            });
        });

        let result = self
            .cloud
            .upload_uri(UploadRequest {
                task_id: UploadTaskId(session_key.clone()),
                source_uri: local.uri.clone(),
                size: local.size,
                dest_uri: task.dest_uri.clone(),
                name: local.name.clone(),
                mime: local.mime.clone(),
                cancel,
                progress,
            })
            .await;

        // The terminal state written below is authoritative: stop the throttled
        // writer first so it cannot overwrite it with a stale sample.
        writer.abort();
        self.cancels.lock().unwrap().remove(&session_key);

        // The attempt may have been cancelled while it ran: its result must not
        // turn a cancelled task back into a finished or failed one.
        if let Some(current) = self.catalog.upload_task(&session_key).await?
            && current.state == UploadState::Cancelled
        {
            return Ok(UploadOutcome {
                task: current,
                asset: None,
            });
        }

        match result {
            Ok(entry) => {
                self.set_state(&mut task, UploadState::Completing).await?;
                let fid = FolderId::new(folder_id);
                let client_exif = (local.date_taken > 0).then_some(ExifData {
                    taken_at: Some(local.date_taken),
                    width: local.width,
                    height: local.height,
                });
                let record = asset_record(&entry, &fid, client_exif.as_ref());
                let summary = record.summary.clone();
                self.catalog.upsert_asset(&record).await?;
                task.state = UploadState::Done;
                task.uploaded = task.total;
                task.speed = 0;
                task.eta_seconds = None;
                task.error = None;
                self.catalog.update_upload(&task).await?;
                Ok(UploadOutcome {
                    task,
                    asset: Some(summary),
                })
            }
            Err(err) => {
                task.state = UploadState::Failed;
                task.error = Some(err.clone());
                task.speed = 0;
                task.eta_seconds = None;
                self.catalog.update_upload(&task).await?;
                Err(err)
            }
        }
    }

    async fn set_state(&self, task: &mut UploadTask, state: UploadState) -> Result<(), AppError> {
        task.state = state;
        self.catalog.update_upload(task).await
    }

    pub async fn pending(&self) -> Result<Vec<UploadTask>, AppError> {
        self.catalog.pending_uploads().await
    }
}
