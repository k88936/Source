//! Connection, login and remote-folder commands.

use std::sync::atomic::Ordering;

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
) -> Result<SiteInfo, AccountError> {
    state.core.account.set_server(&params.base_url).await
}

#[tauri::command]
pub(crate) async fn login(
    app: AppHandle,
    state: State<'_, AppState>,
    params: LoginParams,
) -> Result<LoginOutcome, AccountError> {
    let outcome = state
        .core
        .account
        .login(&params.email, &params.password)
        .await?;
    if let LoginOutcome::Authenticated { account } = &outcome {
        announce_setup(&app, account);
    }
    Ok(outcome)
}

#[tauri::command]
pub(crate) async fn login_2fa(
    app: AppHandle,
    state: State<'_, AppState>,
    params: Login2faParams,
) -> Result<AccountState, AccountError> {
    let account = state
        .core
        .account
        .login_2fa(&params.session_id, &params.otp)
        .await?;
    announce_setup(&app, &account);
    Ok(account)
}

#[tauri::command]
pub(crate) async fn logout(state: State<'_, AppState>) -> Result<(), AccountError> {
    state.core.account.logout().await
}

#[tauri::command]
pub(crate) async fn session_state(
    state: State<'_, AppState>,
) -> Result<SessionState, AccountError> {
    let account = state.core.account.get_account_state().await?;
    let setup = if state.core.account.is_live() {
        SessionSetup::Live
    } else if state.core.restoring.load(Ordering::Relaxed) {
        SessionSetup::Restoring
    } else {
        SessionSetup::Offline
    };
    Ok(SessionState { account, setup })
}

/// Tell the page that a client now stands behind the account.
fn announce_setup(app: &AppHandle, account: &AccountState) {
    let _ = app.emit(
        "session://setup",
        SessionState {
            account: Some(account.clone()),
            setup: SessionSetup::Live,
        },
    );
}

#[tauri::command]
pub(crate) async fn capacity(state: State<'_, AppState>) -> Result<Capacity, AccountError> {
    state.core.account.get_capacity().await
}

#[tauri::command]
pub(crate) async fn list_remote_folders(
    state: State<'_, AppState>,
    params: ListRemoteFoldersParams,
) -> Result<RemoteFolderPage, FolderError> {
    state
        .core
        .folders
        .list_remote_folders(&params.uri, params.page_token.as_deref())
        .await
}

/// Create a folder from inside the album picker.
#[tauri::command]
pub(crate) async fn create_folder(
    state: State<'_, AppState>,
    params: CreateFolderParams,
) -> Result<FolderRef, FolderError> {
    state
        .core
        .folders
        .create_folder(&params.parent_uri, &params.name)
        .await
}
