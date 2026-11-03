//! Connection, login and library-root commands.

use crate::prelude::*;

#[tauri::command]
pub(crate) fn app_info() -> AppInfo {
    AppInfo {
        name: "Cloudreve 相册".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}

#[tauri::command]
pub(crate) async fn set_server(
    state: State<'_, AppState>,
    params: SetServerParams,
) -> Result<SiteInfo, AppError> {
    state.core.set_server(&params.base_url).await
}

#[tauri::command]
pub(crate) async fn login(
    state: State<'_, AppState>,
    params: LoginParams,
) -> Result<LoginOutcome, AppError> {
    state.core.login(&params.email, &params.password).await
}

#[tauri::command]
pub(crate) async fn login_2fa(
    state: State<'_, AppState>,
    params: Login2faParams,
) -> Result<SessionState, AppError> {
    state.core.login_2fa(&params.session_id, &params.otp).await
}

#[tauri::command]
pub(crate) async fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    state.core.logout().await
}

#[tauri::command]
pub(crate) fn session_state(state: State<'_, AppState>) -> Option<SessionState> {
    state.core.session_state()
}

#[tauri::command]
pub(crate) async fn refresh_capacity(state: State<'_, AppState>) -> Result<Capacity, AppError> {
    state.core.refresh_capacity().await
}

#[tauri::command]
pub(crate) async fn list_remote_folders(
    state: State<'_, AppState>,
    params: ListRemoteFoldersParams,
) -> Result<RemoteFolderPage, AppError> {
    state
        .core
        .list_remote_folders(&params.uri, params.page_token.as_deref())
        .await
}

#[tauri::command]
pub(crate) async fn set_library_root(
    state: State<'_, AppState>,
    params: SetLibraryRootParams,
) -> Result<FolderRef, AppError> {
    state.core.set_library_root(&params.uri).await
}
