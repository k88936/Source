//! Device library commands: the browser the picker is built on, the registered
//! local albums, and their direct media files.

use crate::prelude::*;

#[tauri::command]
pub(crate) fn media_permission_state(state: State<'_, AppState>) -> MediaPermissionState {
    state.core.browse.media_permission_state()
}

#[tauri::command]
pub(crate) fn request_media_permission(state: State<'_, AppState>) -> MediaPermissionState {
    state.core.browse.request_media_permission()
}

/// One step of the device directory browser (`$HOME` when `parent` is absent).
#[tauri::command]
pub(crate) fn local_browse(
    state: State<'_, AppState>,
    params: LocalBrowseParams,
) -> Result<LocalBrowsePage, AppError> {
    state.core.albums.local_browse(params.parent.as_deref())
}

#[tauri::command]
pub(crate) async fn local_album_list(
    state: State<'_, AppState>,
) -> Result<Vec<LocalAlbumSummary>, AppError> {
    state.core.albums.local_album_list().await
}

#[tauri::command]
pub(crate) async fn local_album_add(
    state: State<'_, AppState>,
    params: AddLocalAlbumParams,
) -> Result<LocalAlbumSummary, AppError> {
    state.core.albums.add_local_album(params).await
}

#[tauri::command]
pub(crate) async fn local_album_remove(
    state: State<'_, AppState>,
    params: LocalAlbumParams,
) -> Result<(), AppError> {
    state.core.albums.remove_local_album(&params.id).await
}

#[tauri::command]
pub(crate) fn local_asset_page(
    state: State<'_, AppState>,
    params: LocalAssetPageParams,
) -> Result<LocalAssetPage, AppError> {
    state.core.albums.local_asset_page(params)
}

/// Start a sync pass over every registered local album. Returns at once; the
/// pass reports through `sync://run`, and its files reach the upload queue.
#[tauri::command]
pub(crate) async fn sync_start(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Run, AppError> {
    state
        .core
        .sync
        .start_sync(super::run_sink(&app, "sync://run"))
        .await
}

/// Stop the sync pass after the directories already being walked.
#[tauri::command]
pub(crate) async fn sync_stop(state: State<'_, AppState>) -> Result<(), AppError> {
    state.core.sync.stop_sync();
    Ok(())
}

/// The newest sync pass, so a page can restore its progress after a reload.
#[tauri::command]
pub(crate) async fn sync_run(state: State<'_, AppState>) -> Result<Option<Run>, AppError> {
    state.core.sync.sync_run().await
}
