//! Upload queue commands and the upload runner.

use source_core::AppCore;

use crate::prelude::*;
use source_core::dto::UploadProgressEvent;

#[tauri::command]
pub(crate) async fn enqueue_uploads(
    app: AppHandle,
    state: State<'_, AppState>,
    params: EnqueueUploadsParams,
) -> Result<Vec<UploadTask>, AppError> {
    let tasks = state.core.enqueue_uploads(params).await?;
    let ids = tasks.iter().map(|t| t.id.0.clone()).collect::<Vec<_>>();
    let core = Arc::clone(&state.core);
    tauri::async_runtime::spawn(async move {
        run_upload_batch(app, core, ids).await;
    });
    Ok(tasks)
}

#[tauri::command]
pub(crate) async fn upload_tasks(state: State<'_, AppState>) -> Result<Vec<UploadTask>, AppError> {
    state.core.upload_tasks().await
}

#[tauri::command]
pub(crate) async fn cancel_upload(
    state: State<'_, AppState>,
    params: UploadTaskParams,
) -> Result<(), AppError> {
    state.core.cancel_upload(&params.task_id.0).await
}

#[tauri::command]
pub(crate) async fn retry_upload(
    app: AppHandle,
    state: State<'_, AppState>,
    params: UploadTaskParams,
) -> Result<UploadTask, AppError> {
    let task = state.core.retry_upload(&params.task_id.0).await?;
    let core = Arc::clone(&state.core);
    let id = task.id.0.clone();
    tauri::async_runtime::spawn(async move {
        run_upload_batch(app, core, vec![id]).await;
    });
    Ok(task)
}

/// How many uploads run at once.
///
/// Two keeps a phone uplink saturated without letting one huge video starve a
/// batch of small photos.
const UPLOAD_CONCURRENCY: usize = 2;

/// Automatic retries after the first failure; the first wait is
/// [`RETRY_BASE_DELAY`], then it doubles.
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY: std::time::Duration = std::time::Duration::from_secs(3);

/// Backoff before retry number `attempt` (1-based).
fn retry_delay(attempt: u32) -> std::time::Duration {
    let factor = 1u32 << attempt.saturating_sub(1).min(4);
    RETRY_BASE_DELAY * factor
}

pub(crate) async fn run_upload_batch(app: AppHandle, core: Arc<AppCore>, ids: Vec<String>) {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(UPLOAD_CONCURRENCY));
    let mut set = tokio::task::JoinSet::new();
    for id in ids {
        let app = app.clone();
        let core = Arc::clone(&core);
        let semaphore = Arc::clone(&semaphore);
        set.spawn(async move {
            run_one_upload(app, core, id, semaphore).await;
        });
    }
    while set.join_next().await.is_some() {}
}

async fn run_one_upload(
    app: AppHandle,
    core: Arc<AppCore>,
    id: String,
    semaphore: Arc<tokio::sync::Semaphore>,
) {
    let mut retries = 0u32;
    loop {
        // Take a slot per attempt, so a task sleeping in backoff does not hold
        // one of the few concurrent upload slots.
        let attempt = {
            let Ok(_permit) = semaphore.clone().acquire_owned().await else {
                return;
            };
            attempt_upload(&app, &core, &id).await
        };
        match attempt {
            Ok(outcome) => {
                let _ = app.emit(
                    "upload://state",
                    UploadStateEvent {
                        task_id: outcome.task.id,
                        state: outcome.task.state,
                        error: None,
                    },
                );
                if let Some(asset) = outcome.asset {
                    // One tile, one album count bump — the UI patches instead
                    // of reloading the timeline.
                    let _ = app.emit(
                        "catalog://changed",
                        CatalogChangedEvent {
                            kind: CatalogChangeKind::Assets,
                            assets: vec![asset],
                            albums: Vec::new(),
                        },
                    );
                }
                return;
            }
            Err(error) => {
                // Transient failures resume from the journal on the next try;
                // anything else (auth, permission, conflict) is final.
                if error.retryable && retries < MAX_RETRIES {
                    retries += 1;
                    tokio::time::sleep(retry_delay(retries)).await;
                    if core.requeue_upload(&id).await.is_err() {
                        return;
                    }
                    continue;
                }
                let _ = app.emit(
                    "upload://state",
                    UploadStateEvent {
                        task_id: id.clone().into(),
                        state: UploadState::Failed,
                        error: Some(error),
                    },
                );
                return;
            }
        }
    }
}

/// One attempt: announce the queued state, forward progress, run the upload.
async fn attempt_upload(
    app: &AppHandle,
    core: &Arc<AppCore>,
    id: &str,
) -> Result<source_core::app::UploadOutcome, AppError> {
    let queued = core
        .upload_task(id)
        .await
        .ok()
        .flatten()
        .map(|task| UploadStateEvent {
            task_id: task.id,
            state: UploadState::Queued,
            error: None,
        });
    if let Some(event) = queued {
        let _ = app.emit("upload://state", &event);
    }
    let progress_app = app.clone();
    let progress: Arc<dyn Fn(UploadProgressEvent) + Send + Sync> =
        Arc::new(move |event: UploadProgressEvent| {
            let _ = progress_app.emit("upload://progress", &event);
        });
    core.run_upload(id, progress).await
}
