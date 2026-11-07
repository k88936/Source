//! Tauri commands, grouped by the part of the app they serve.

use source_core::app::SetJobsPausedParams;

use crate::prelude::*;
use crate::state::JOBS_PAUSED_KEY;

pub(crate) mod catalog;
pub(crate) mod connection;
pub(crate) mod export;
pub(crate) mod local;
pub(crate) mod uploads;

/// The board: every job this process ran, newest first.
///
/// One list for every kind of work, so a page picks the rows it cares about by
/// key instead of subscribing to an event per kind.
#[tauri::command]
pub(crate) async fn job_list(state: State<'_, AppState>) -> Result<Vec<Job>, AppError> {
    Ok(state.core.jobs.list())
}

/// Whether background work is paused, for the settings page to restore.
#[tauri::command]
pub(crate) async fn jobs_paused(state: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(state.core.jobs.is_paused())
}

/// Pause or resume background work, and remember it for the next launch.
#[tauri::command]
pub(crate) async fn set_jobs_paused(
    state: State<'_, AppState>,
    params: SetJobsPausedParams,
) -> Result<(), AppError> {
    state
        .core
        .settings
        .set_setting(JOBS_PAUSED_KEY, if params.paused { "1" } else { "0" })
        .await?;
    if params.paused {
        state.core.jobs.pause();
    } else {
        state.core.jobs.resume();
    }
    // The switch is the only thing that decides whether Android keeps this
    // process up: a paused queue has no reason to hold it, and a resumed one is
    // held before the first job arrives, not after it.
    #[cfg(target_os = "android")]
    crate::android_keep_alive::sync(!params.paused).await;
    Ok(())
}
