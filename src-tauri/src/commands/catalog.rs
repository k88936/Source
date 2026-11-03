//! Timeline, album and asset commands.

use crate::prelude::*;

#[tauri::command]
pub(crate) async fn timeline_page(
    state: State<'_, AppState>,
    params: TimelinePageParams,
) -> Result<TimelinePage, AppError> {
    state.core.timeline_page(params).await
}

#[tauri::command]
pub(crate) async fn album_list(state: State<'_, AppState>) -> Result<Vec<AlbumSummary>, AppError> {
    state.core.album_list().await
}

#[tauri::command]
pub(crate) async fn album_page(
    state: State<'_, AppState>,
    params: AlbumPageParams,
) -> Result<TimelinePage, AppError> {
    state.core.album_page(params).await
}

#[tauri::command]
pub(crate) async fn asset_detail(
    state: State<'_, AppState>,
    params: AssetDetailParams,
) -> Result<AssetDetail, AppError> {
    state.core.asset_detail(&params.asset_id.0).await
}

#[tauri::command]
pub(crate) async fn search_assets(
    state: State<'_, AppState>,
    params: SearchParams,
) -> Result<TimelinePage, AppError> {
    state.core.search(params).await
}

#[tauri::command]
pub(crate) async fn set_favorite(
    app: AppHandle,
    state: State<'_, AppState>,
    params: SetFavoriteParams,
) -> Result<AssetSummary, AppError> {
    let asset = state
        .core
        .set_favorite(&params.asset_id.0, params.value)
        .await?;
    emit_assets(&app, vec![asset.clone()]);
    Ok(asset)
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
        .move_assets_to_album(&ids, &params.album_id.0)
        .await?;
    // Counts and mosaics of both the source and the target album changed.
    if let Ok(albums) = state.core.album_list().await {
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

#[tauri::command]
pub(crate) async fn set_album_cover(
    app: AppHandle,
    state: State<'_, AppState>,
    params: SetAlbumCoverParams,
) -> Result<AlbumSummary, AppError> {
    let album = state
        .core
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
        .set_album_description(&params.album_id.0, &params.description)
        .await?;
    emit_albums(&app, vec![album.clone()]);
    Ok(album)
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
