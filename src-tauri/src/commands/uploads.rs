//! Upload queue commands.
//!
//! The queue owns its own scheduling, so a command only changes rows and hands
//! the ids it cares about to it; progress and state changes arrive as events.

use crate::prelude::*;

#[tauri::command]
pub(crate) async fn enqueue_uploads(
    state: State<'_, AppState>,
    params: EnqueueUploadsParams,
) -> Result<Vec<UploadTask>, AppError> {
    let tasks = state.core.uploads.enqueue_files(params).await?;
    state
        .core
        .uploads
        .start(tasks.iter().map(|task| task.id.clone()).collect());
    Ok(tasks)
}

#[tauri::command]
pub(crate) async fn upload_tasks(state: State<'_, AppState>) -> Result<Vec<UploadTask>, AppError> {
    state.core.uploads.list_tasks().await
}

#[tauri::command]
pub(crate) async fn cancel_upload(
    state: State<'_, AppState>,
    params: UploadTaskParams,
) -> Result<(), AppError> {
    state.core.uploads.cancel(&params.task_id).await
}

#[tauri::command]
pub(crate) async fn retry_upload(
    state: State<'_, AppState>,
    params: UploadTaskParams,
) -> Result<UploadTask, AppError> {
    let task = state.core.uploads.retry(&params.task_id).await?;
    state.core.uploads.start(vec![task.id.clone()]);
    Ok(task)
}
