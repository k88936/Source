//! Device library commands.

use crate::prelude::*;

#[tauri::command]
pub(crate) fn media_permission_state(state: State<'_, AppState>) -> MediaPermissionState {
    state.core.browse.media_permission_state()
}

#[tauri::command]
pub(crate) fn request_media_permission(state: State<'_, AppState>) -> MediaPermissionState {
    state.core.browse.request_media_permission()
}

#[tauri::command]
pub(crate) fn local_album_list(state: State<'_, AppState>) -> Result<Vec<LocalAlbum>, AppError> {
    state.core.browse.local_album_list()
}

#[tauri::command]
pub(crate) fn local_asset_page(
    state: State<'_, AppState>,
    params: LocalAssetPageParams,
) -> Result<LocalAssetPage, AppError> {
    state.core.browse.local_asset_page(params)
}
