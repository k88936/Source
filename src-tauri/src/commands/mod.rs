//! Tauri commands, grouped by the part of the app they serve.

use crate::prelude::*;

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
