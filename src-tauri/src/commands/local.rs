//! Device library commands: the browser the picker is built on, the registered
//! local albums, and their direct media files.

use crate::prelude::*;

#[tauri::command]
pub(crate) async fn media_permission_state(
    state: State<'_, AppState>,
) -> Result<MediaPermissionState, BrowseError> {
    state.core.browse.get_media_permission_state().await
}

#[tauri::command]
pub(crate) async fn request_media_permission(
    state: State<'_, AppState>,
) -> Result<MediaPermissionState, BrowseError> {
    state.core.browse.request_media_permission().await
}

/// One step of the device library browser: `$HOME` on desktop, every bucket on
/// Android, where asking also raises the system permission dialog.
#[tauri::command]
pub(crate) async fn local_browse(
    state: State<'_, AppState>,
    params: LocalBrowseParams,
) -> Result<LocalBrowsePage, AlbumError> {
    state
        .core
        .albums
        .browse_local(params.parent.as_deref())
        .await
}

#[tauri::command]
pub(crate) async fn local_album_list(
    state: State<'_, AppState>,
) -> Result<Vec<LocalAlbumSummary>, AlbumError> {
    state.core.albums.local_album_list().await
}

#[tauri::command]
pub(crate) async fn local_album_add(
    state: State<'_, AppState>,
    params: AddLocalAlbumParams,
) -> Result<LocalAlbumSummary, AlbumError> {
    state.core.albums.add_local_album(params).await
}

#[tauri::command]
pub(crate) async fn local_album_remove(
    state: State<'_, AppState>,
    params: LocalAlbumParams,
) -> Result<(), AlbumError> {
    state.core.albums.remove_local_album(&params.id).await
}

#[tauri::command]
pub(crate) async fn local_asset_page(
    state: State<'_, AppState>,
    params: LocalAssetPageParams,
) -> Result<LocalAssetPage, AlbumError> {
    state.core.albums.get_local_asset_page(params).await
}

/// Queue one scan per registered local album and return how many. Each scan
/// offers its files to the upload queue, which runs them as jobs of their own.
#[tauri::command]
pub(crate) async fn sync_start(state: State<'_, AppState>) -> Result<usize, SyncError> {
    state.core.sync.sync_all().await
}

/// Stop the scan: what is still waiting is dropped, what is walking ends after
/// the page it is on.
#[tauri::command]
pub(crate) async fn sync_stop(state: State<'_, AppState>) -> Result<(), SyncError> {
    state.core.sync.stop();
    Ok(())
}
