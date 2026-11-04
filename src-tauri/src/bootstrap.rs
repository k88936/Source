//! Startup wiring: the composition root and resuming queued uploads.

use std::sync::Arc;

use source_core::{
    AccountService, AlbumsService, AppError, BrowseService, CloudreveAuth, CloudreveStorage,
    CredentialStore, FoldersService, FsLocalMedia, SqliteCatalog, ThumbCache, UploadSessionPort,
    UploadsService,
};
use tauri::Manager;

use crate::commands::uploads::run_upload_batch;
#[cfg(not(debug_assertions))]
use crate::credentials::KeyringCredentialStore;
use crate::state::{App, AppState};

/// Open storage, build every service, and resume uploads left queued by a crash.
///
/// This is the register section and the only place an adapter is constructed.
/// Auth and storage share one live account, so a login is visible to all of them
/// at once; a second graph (Android media, an e2e harness) swaps an adapter by
/// changing a type argument here, not by touching the use cases.
///
/// Migrations run inside `SqliteCatalog::open`, so the catalog is ready before
/// anything can touch it.
pub(crate) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data)?;

    let core: Arc<App> = Arc::new(tauri::async_runtime::block_on(async {
        let catalog = SqliteCatalog::open(app_data.join("catalog.sqlite3")).await?;
        let sessions: Arc<dyn UploadSessionPort> = Arc::new(catalog.clone());
        let cred_store: Arc<dyn CredentialStore> = {
            #[cfg(debug_assertions)]
            {
                crate::dev::credential_store(&app_data)
            }
            #[cfg(not(debug_assertions))]
            {
                Arc::new(KeyringCredentialStore::new(app.handle().clone()))
            }
        };
        let auth = CloudreveAuth::new(cred_store);
        let cloud = CloudreveStorage::new(Arc::new(auth.clone()), sessions);
        let local = FsLocalMedia::new();
        Ok::<_, AppError>(App {
            account: Arc::new(AccountService::new(auth, cloud.clone(), catalog.clone())),
            folders: Arc::new(FoldersService::new(cloud.clone(), catalog.clone())),
            albums: Arc::new(AlbumsService::new(
                cloud.clone(),
                catalog.clone(),
                local.clone(),
            )),
            browse: Arc::new(BrowseService::new(
                cloud.clone(),
                catalog.clone(),
                local.clone(),
            )),
            uploads: Arc::new(UploadsService::new(cloud, catalog, local)),
        })
    })?);
    let _ = tauri::async_runtime::block_on(core.account.bootstrap());

    #[cfg(debug_assertions)]
    crate::dev::autoconnect(&core);

    let cache = Arc::new(ThumbCache::default_cache(&app_data)?);

    let pending = tauri::async_runtime::block_on(core.uploads.pending()).unwrap_or_default();
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
