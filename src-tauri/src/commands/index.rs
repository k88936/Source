//! Catalog index commands and progress events.

use crate::prelude::*;

#[tauri::command]
pub(crate) async fn start_index(
    app: AppHandle,
    state: State<'_, AppState>,
    params: StartIndexParams,
) -> Result<IndexStatus, AppError> {
    let core = Arc::clone(&state.core);
    let status = core.index.start(params.clone()).await?;
    if status.phase == IndexPhase::Scanning {
        let app_handle = app.clone();
        let core_task = Arc::clone(&core);
        tauri::async_runtime::spawn(async move {
            let monitor_app = app_handle.clone();
            let monitor_core = Arc::clone(&core_task);
            let monitor = tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                    let snapshot = monitor_core.index.status();
                    let _ = monitor_app.emit("catalog://index", &snapshot);
                    if snapshot.phase != IndexPhase::Scanning {
                        break;
                    }
                }
            });
            core_task.index.run(params.full).await;
            let _ = monitor.await;
            let _ = app_handle.emit("catalog://index", core_task.index.status());
            let _ = app_handle.emit(
                "catalog://changed",
                CatalogChangedEvent {
                    kind: CatalogChangeKind::Folders,
                    assets: Vec::new(),
                    albums: Vec::new(),
                },
            );
        });
    }
    Ok(status)
}

#[tauri::command]
pub(crate) fn index_status(state: State<'_, AppState>) -> IndexStatus {
    state.core.index.status()
}

/// Stop after the folder currently being scanned; the pass stays resumable.
#[tauri::command]
pub(crate) fn cancel_index(state: State<'_, AppState>) -> IndexStatus {
    state.core.index.cancel_pass()
}
