//! Startup wiring: catalog, core services and resuming queued uploads.

use std::sync::Arc;

use source_catalog::SqliteCatalog;
use source_cloudreve::CloudreveService;
use source_core::AppCore;
use source_core::UploadSessionPort;
use source_media::{FsLocalMedia, ThumbCache};
use tauri::Manager;

use crate::commands::uploads::run_upload_batch;
#[cfg(not(debug_assertions))]
use crate::credentials::KeyringCredentialStore;
use crate::state::AppState;

/// Open storage, build the core, and resume uploads left queued by a crash.
pub(crate) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data)?;

    // Migrations run inside `open`, so the catalog must be ready before
    // anything touches it.
    let catalog = Arc::new(tauri::async_runtime::block_on(async {
        SqliteCatalog::open(app_data.join("catalog.sqlite3")).await
    })?);
    let sessions: Arc<dyn UploadSessionPort> = catalog.clone();
    let store: Arc<dyn source_cloudreve::CredentialStore> = {
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
    let cloud = Arc::new(CloudreveService::new(store).with_session_store(sessions));
    let local = Arc::new(FsLocalMedia::new(FsLocalMedia::default_roots()));
    let core = Arc::new(AppCore::new(cloud, catalog, local));
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
