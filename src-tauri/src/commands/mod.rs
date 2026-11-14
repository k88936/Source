//! Tauri commands, grouped by the part of the app they serve.

use crate::prelude::*;

pub(crate) mod catalog;
pub(crate) mod connection;
pub(crate) mod export;
pub(crate) mod local;
pub(crate) mod subscription;
pub(crate) mod uploads;

/// Everything the settings page shows.
#[tauri::command]
pub(crate) async fn settings(state: State<'_, AppState>) -> Result<Settings, CatalogError> {
    state.core.settings.get().await
}

/// What the page changed, and nothing else.
///
/// A delta is what makes one screen able to change one setting: a field it does
/// not send is a setting it does not own. It answers nothing, so the page reads
/// [`settings`] again instead of being told what its own write means.
#[tauri::command]
pub(crate) async fn set_settings(
    state: State<'_, AppState>,
    params: SettingsDelta,
) -> Result<(), CatalogError> {
    state.core.settings.set(&params).await?;
    // Holding the process in the background is a platform service on Android
    // and nothing on desktop, so the switch reaches it here rather than in core.
    #[cfg(target_os = "android")]
    if let Some(hold) = params.keep_alive {
        tauri_plugin_keep_alive::sync(hold).await;
    }
    Ok(())
}

/// Which job a card's button is about.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JobCancelParams {
    pub(crate) kind: JobKind,
    pub(crate) id: String,
}

/// Stop one job by the key its row carries.
///
/// What is still waiting is dropped, what is running ends after the item it is
/// on, and the row stays in the list as the state it reached - the page sees
/// the same row stop rather than vanish.
#[tauri::command]
pub(crate) fn job_cancel(state: State<'_, AppState>, params: JobCancelParams) {
    state
        .core
        .jobs
        .cancel_item(params.kind, &JobId::from(params.id));
}
