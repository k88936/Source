//! Timeline, album and asset commands.

use crate::prelude::*;
use crate::protocols::media_address;

#[tauri::command]
pub(crate) async fn timeline_page(
    state: State<'_, AppState>,
    params: TimelinePageParams,
) -> Result<TimelinePage, AppError> {
    state.core.browse.get_timeline_page(params).await
}

#[tauri::command]
pub(crate) async fn asset_detail(
    state: State<'_, AppState>,
    params: AssetDetailParams,
) -> Result<AssetDetail, AppError> {
    state.core.browse.get_asset_detail(&params.asset_id.0).await
}

#[tauri::command]
pub(crate) async fn search_assets(
    state: State<'_, AppState>,
    params: SearchParams,
) -> Result<TimelinePage, AppError> {
    state.core.browse.search(params).await
}

#[tauri::command]
pub(crate) async fn set_favorite(
    app: AppHandle,
    state: State<'_, AppState>,
    params: SetFavoriteParams,
) -> Result<AssetSummary, AppError> {
    let asset = state
        .core
        .browse
        .set_favorite(&params.asset_id.0, params.value)
        .await?;
    emit_assets(&app, vec![asset.clone()]);
    Ok(asset)
}

#[tauri::command]
pub(crate) async fn album_list(state: State<'_, AppState>) -> Result<Vec<AlbumSummary>, AppError> {
    state.core.albums.list_albums().await
}

#[tauri::command]
pub(crate) async fn album_get(
    state: State<'_, AppState>,
    params: AlbumParams,
) -> Result<Option<AlbumSummary>, AppError> {
    state.core.albums.get_album(&params.album_id.0).await
}

#[tauri::command]
pub(crate) async fn album_page(
    state: State<'_, AppState>,
    params: AlbumPageParams,
) -> Result<TimelinePage, AppError> {
    state.core.albums.get_album_page(params).await
}

/// Add a folder as an album. The folder is listed once right away, so a fresh
/// album is never shown empty while the server has media in it.
#[tauri::command]
pub(crate) async fn album_add(
    state: State<'_, AppState>,
    params: AddAlbumParams,
) -> Result<AlbumSummary, AppError> {
    state.core.albums.add_album(params).await
}

/// Forget an album. The folder on the server stays untouched.
#[tauri::command]
pub(crate) async fn album_remove(
    state: State<'_, AppState>,
    params: AlbumParams,
) -> Result<(), AppError> {
    state.core.albums.remove_album(&params.album_id.0).await
}

#[tauri::command]
pub(crate) async fn album_refresh(
    state: State<'_, AppState>,
    params: AlbumParams,
) -> Result<AlbumSummary, AppError> {
    state.core.albums.refresh_album(&params.album_id.0).await
}

/// Start a pass over every album. Returns at once; the pass reports through
/// `index://run`, and every refreshed album is visible on its own row.
#[tauri::command]
pub(crate) async fn index_start(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Run, AppError> {
    state
        .core
        .albums
        .start_index(super::run_sink(&app, "index://run"))
        .await
}

/// Stop the index pass after the albums already being listed.
#[tauri::command]
pub(crate) async fn index_stop(state: State<'_, AppState>) -> Result<(), AppError> {
    state.core.albums.stop_index();
    Ok(())
}

/// The newest index pass, so a page can restore its progress after a reload.
#[tauri::command]
pub(crate) async fn index_run(state: State<'_, AppState>) -> Result<Option<Run>, AppError> {
    state.core.albums.get_index_run().await
}

#[tauri::command]
pub(crate) async fn set_album_cover(
    app: AppHandle,
    state: State<'_, AppState>,
    params: SetAlbumCoverParams,
) -> Result<AlbumSummary, AppError> {
    let album = state
        .core
        .albums
        .set_album_cover(&params.album_id.0, &params.asset_id.0)
        .await?;
    emit_albums(&app, vec![album.clone()]);
    Ok(album)
}

#[tauri::command]
pub(crate) async fn set_album_description(
    app: AppHandle,
    state: State<'_, AppState>,
    params: SetAlbumDescriptionParams,
) -> Result<AlbumSummary, AppError> {
    let album = state
        .core
        .albums
        .set_album_description(&params.album_id.0, &params.description)
        .await?;
    emit_albums(&app, vec![album.clone()]);
    Ok(album)
}

/// Move the selection into an album. This is destructive: Cloudreve albums are
/// folders, so the files leave their current folder.
#[tauri::command]
pub(crate) async fn move_assets_to_album(
    app: AppHandle,
    state: State<'_, AppState>,
    params: MoveAssetsParams,
) -> Result<Vec<AssetSummary>, AppError> {
    let ids = params
        .asset_ids
        .into_iter()
        .map(|asset| asset.0)
        .collect::<Vec<_>>();
    let moved = state
        .core
        .albums
        .move_assets_to_album(&ids, &params.album_id.0)
        .await?;
    // Counts and mosaics of both the source and the target album changed.
    if let Ok(albums) = state.core.albums.list_albums().await {
        let _ = app.emit(
            "catalog://changed",
            CatalogChangedEvent {
                kind: CatalogChangeKind::Albums,
                assets: Vec::new(),
                albums,
            },
        );
    }
    Ok(moved)
}

fn emit_assets(app: &AppHandle, assets: Vec<AssetSummary>) {
    let _ = app.emit(
        "catalog://changed",
        CatalogChangedEvent {
            kind: CatalogChangeKind::Assets,
            assets,
            albums: Vec::new(),
        },
    );
}

fn emit_albums(app: &AppHandle, albums: Vec<AlbumSummary>) {
    let _ = app.emit(
        "catalog://changed",
        CatalogChangedEvent {
            kind: CatalogChangeKind::Albums,
            assets: Vec::new(),
            albums,
        },
    );
}

/// Every address a media element loads: thumbnails, full-size stills, video and
/// files outside the catalog. The adapter owns the scheme, the route namespace
/// and the platform differences, so no component builds a URL itself.
#[tauri::command]
pub(crate) async fn media_source(
    state: State<'_, AppState>,
    params: MediaSourceParams,
) -> Result<String, AppError> {
    match params {
        MediaSourceParams::Thumb { asset_id } => Ok(media_address("thumb", &asset_id.0)),
        MediaSourceParams::Original { asset_id } => Ok(media_address("original", &asset_id.0)),
        MediaSourceParams::LocalThumb { uri } => Ok(media_address("local", &uri)),
        MediaSourceParams::Video { asset_id } => video_address(&state, &asset_id.0).await,
    }
}

/// WebKitGTK's media player only accepts `http(s)://`, so desktop plays the
/// file from a signed server URL.
#[cfg(desktop)]
async fn video_address(state: &AppState, asset_id: &str) -> Result<String, AppError> {
    state.core.browse.get_media_url(asset_id).await
}

/// Android's player is fine with the custom scheme, which the webview
/// intercepts before it reaches the network stack.
#[cfg(mobile)]
async fn video_address(_state: &AppState, asset_id: &str) -> Result<String, AppError> {
    Ok(media_address("original", asset_id))
}
