//! Tauri commands, grouped by the part of the app they serve.

use source_core::app::SetKeepAliveParams;

use crate::prelude::*;
use crate::state::KEEP_ALIVE_KEY;

pub(crate) mod catalog;
pub(crate) mod connection;
pub(crate) mod export;
pub(crate) mod local;
pub(crate) mod subscription;
pub(crate) mod uploads;

/// The board: every job this process ran, newest first.
///
/// One list for every kind of work, so a page picks the rows it cares about by
/// key instead of subscribing to an event per kind.
#[tauri::command]
pub(crate) async fn job_list(state: State<'_, AppState>) -> Result<Vec<Job>, CatalogError> {
    Ok(state.core.jobs.list())
}

/// Whether the settings page asks the platform to hold the process, for it to
/// restore.
#[tauri::command]
pub(crate) async fn keep_alive(state: State<'_, AppState>) -> Result<bool, CatalogError> {
    let stored = state.core.settings.get_setting(KEEP_ALIVE_KEY).await?;
    Ok(stored.as_deref() != Some("0"))
}

/// Hold the process in the background or let the platform kill it, and remember
/// the choice for the next launch.
///
/// Only Android acts on it; the job queue is not involved.
#[tauri::command]
pub(crate) async fn set_keep_alive(
    state: State<'_, AppState>,
    params: SetKeepAliveParams,
) -> Result<(), CatalogError> {
    state
        .core
        .settings
        .set_setting(KEEP_ALIVE_KEY, if params.hold { "1" } else { "0" })
        .await?;
    // The service starts before any work shows up, and stops before the
    // platform notices the process has nothing to do.
    #[cfg(target_os = "android")]
    tauri_plugin_keep_alive::sync(params.hold).await;
    Ok(())
}
