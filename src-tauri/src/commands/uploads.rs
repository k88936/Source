//! Upload queue commands.
//!
//! The queue owns its own scheduling, so a command only changes rows: every row
//! it writes is offered to the job queue, which decides when it runs.

use crate::prelude::*;

#[tauri::command]
pub(crate) async fn enqueue_uploads(
    state: State<'_, AppState>,
    params: EnqueueUploadsParams,
) -> Result<Vec<QueuedUpload>, UploadError> {
    state.core.uploads.enqueue_files(params).await
}

#[tauri::command]
pub(crate) async fn upload_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<QueuedUpload>, UploadError> {
    state.core.uploads.list_tasks().await
}

#[tauri::command]
pub(crate) async fn cancel_upload(
    state: State<'_, AppState>,
    params: UploadTaskParams,
) -> Result<(), UploadError> {
    state.core.uploads.cancel(&params.task_id).await
}

#[tauri::command]
pub(crate) async fn retry_upload(
    state: State<'_, AppState>,
    params: UploadTaskParams,
) -> Result<QueuedUpload, UploadError> {
    state.core.uploads.retry(&params.task_id).await
}
