//! Startup wiring: the composition root and resuming queued uploads.

use std::path::Path;
use std::sync::Arc;

use source_catalog::SqliteCatalog;
use source_cloudreve::{CloudreveService, CredentialStore};
use source_core::{AppCore, AppError, UploadSessionPort};
use source_media::{FsLocalMedia, ThumbCache};
use tauri::Manager;

use crate::commands::uploads::run_upload_batch;
#[cfg(not(debug_assertions))]
use crate::credentials::KeyringCredentialStore;
use crate::state::{App, AppState};

/// Build the dependency graph.
///
/// This is the register section: the only place an adapter is constructed, and
/// the concrete types are fixed by [`App`]. A second graph (Android media, an
/// e2e harness) swaps an adapter by changing that alias and calling this again,
/// not by touching the use cases.
///
/// Migrations run inside `SqliteCatalog::open`, so the catalog is ready before
/// anything can touch it.
pub(crate) async fn compose(
    app_data: &Path,
    store: Arc<dyn CredentialStore>,
) -> Result<App, AppError> {
    let catalog = SqliteCatalog::open(app_data.join("catalog.sqlite3")).await?;
    let sessions: Arc<dyn UploadSessionPort> = Arc::new(catalog.clone());
    let cloud = CloudreveService::new(store, sessions);
    let local = FsLocalMedia::new(FsLocalMedia::default_roots());
    Ok(AppCore::new(cloud, catalog, local))
}

/// Open storage, build the core, and resume uploads left queued by a crash.
pub(crate) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data)?;

    let store: Arc<dyn CredentialStore> = {
        #[cfg(debug_assertions)]
        {
            crate::dev::credential_store(&app_data)
        }
        #[cfg(not(debug_assertions))]
        {
            Arc::new(KeyringCredentialStore::new(
                app.handle().clone(),
                app_data.join("credentials.json"),
            ))
        }
    };
    let core: Arc<App> = Arc::new(tauri::async_runtime::block_on(compose(&app_data, store))?);
    let _ = tauri::async_runtime::block_on(core.bootstrap());

    #[cfg(debug_assertions)]
    crate::dev::autoconnect(&core);

    let cache = Arc::new(ThumbCache::default_cache(&app_data)?);

    let pending = tauri::async_runtime::block_on(core.pending_uploads()).unwrap_or_default();
    let pending_ids = pending.into_iter().map(|t| t.id.0).collect::<Vec<_>>();
    if !pending_ids.is_empty() {
        let handle = app.handle().clone();
        let resume_core = Arc::clone(&core);
        tauri::async_runtime::spawn(async move {
            run_upload_batch(handle, resume_core, pending_ids).await;
        });
    }

    app.manage(AppState {
        core,
        cache,
        app_data,
    });
    Ok(())
}
