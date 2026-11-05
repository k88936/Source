//! Startup wiring: the composition root and resuming what a crash left behind.

use std::sync::Arc;

use source_core::app::uploads::{UploadProgressEvent, UploadSinks, UploadStateEvent};
use source_core::app::{CatalogChangeKind, CatalogChangedEvent};
use source_core::store::AssetSummary;
use source_core::{
    AccountService, AlbumsService, AppError, BrowseService, CloudreveAuth, CloudreveStorage,
    CredentialStore, FoldersService, RunStore, SqliteCatalog, SyncService, ThumbCache,
    UploadSessionPort, UploadsService,
};
use tauri::{Emitter, Manager};

#[cfg(not(debug_assertions))]
use crate::credentials::KeyringCredentialStore;
use crate::state::{App, AppState, LocalMedia};

/// Open storage, build every service, and pick up what a crash left behind.
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
    let sinks = upload_sinks(app.handle());

    let core: Arc<App> = Arc::new(tauri::async_runtime::block_on(async {
        let catalog = SqliteCatalog::open(app_data.join("catalog.sqlite3")).await?;
        // A process that died cannot still be busy: close its passes before
        // anything reads them as live.
        catalog.interrupt_runs().await?;
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
        let local = LocalMedia::new();
        let uploads = Arc::new(UploadsService::new(
            cloud.clone(),
            catalog.clone(),
            local.clone(),
            sinks,
        ));
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
            sync: Arc::new(SyncService::new(catalog, local, Arc::clone(&uploads))),
            uploads,
        })
    })?);
    let _ = tauri::async_runtime::block_on(core.account.bootstrap());

    #[cfg(debug_assertions)]
    crate::dev::autoconnect(&core);

    let cache = Arc::new(ThumbCache::default_cache(&app_data)?);

    // Uploads a previous process left behind resume from their sessions. The
    // work runs as a task on Tauri's runtime, not inside a one-off block: the
    // batch it starts has to outlive setup, and recovering twenty thousand rows
    // must not hold the window.
    {
        let uploads = Arc::clone(&core.uploads);
        tauri::async_runtime::spawn(async move {
            let pending = uploads.recover().await.unwrap_or_default();
            uploads.start(pending.into_iter().map(|task| task.id).collect());
        });
    }

    app.manage(AppState {
        core,
        cache,
        app_data,
    });
    Ok(())
}

/// Where the upload queue reports: the webview's events.
fn upload_sinks(handle: &tauri::AppHandle) -> UploadSinks {
    UploadSinks {
        progress: {
            let handle = handle.clone();
            Arc::new(move |event: UploadProgressEvent| {
                let _ = handle.emit("upload://progress", &event);
            })
        },
        state: {
            let handle = handle.clone();
            Arc::new(move |event: &UploadStateEvent| {
                let _ = handle.emit("upload://state", event);
            })
        },
        uploaded: {
            let handle = handle.clone();
            Arc::new(move |asset: &AssetSummary| {
                // One tile, one album count bump - the UI patches instead of
                // reloading the timeline.
                let _ = handle.emit(
                    "catalog://changed",
                    CatalogChangedEvent {
                        kind: CatalogChangeKind::Assets,
                        assets: vec![asset.clone()],
                        albums: Vec::new(),
                    },
                );
            })
        },
    }
}
