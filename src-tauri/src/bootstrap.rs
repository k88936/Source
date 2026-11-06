//! Startup wiring: the composition root and resuming what a crash left behind.

use std::sync::Arc;

use source_core::app::uploads::{UploadProgressEvent, UploadSinks, UploadStateEvent};
use source_core::app::{CatalogChangeKind, CatalogChangedEvent};
use source_core::store::AssetSummary;
use source_core::{
    AccountService, AlbumsService, AppError, BrowseService, CloudreveAuth, CloudreveStorage,
    CredentialStore, FoldersService, SqliteCatalog, SyncService, ThumbCache, UploadSessionPort,
    UploadsService,
};
use source_jobs::Jobs;
use tauri::{Emitter, Manager};

#[cfg(not(debug_assertions))]
use crate::credentials::KeyringCredentialStore;
use crate::state::{App, AppState, LocalMedia};

/// How many jobs run at once, over every kind of work.
///
/// One number for the whole process, so nothing can raise it by starting one
/// more batch: a scan and the uploads it offered share it, and the queue decides
/// which job goes next.
const JOB_CONCURRENCY: usize = 3;

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
        let jobs = Jobs::new(JOB_CONCURRENCY);
        let uploads = Arc::new(UploadsService::new(
            cloud.clone(),
            catalog.clone(),
            local.clone(),
            sinks,
            jobs.clone(),
        ));
        Ok::<_, AppError>(App {
            account: Arc::new(AccountService::new(auth, cloud.clone(), catalog.clone())),
            folders: Arc::new(FoldersService::new(cloud.clone(), catalog.clone())),
            albums: Arc::new(AlbumsService::new(
                cloud.clone(),
                catalog.clone(),
                local.clone(),
                jobs.clone(),
            )),
            browse: Arc::new(BrowseService::new(
                cloud.clone(),
                catalog.clone(),
                local.clone(),
            )),
            sync: Arc::new(SyncService::new(
                catalog,
                local,
                Arc::clone(&uploads),
                jobs.clone(),
            )),
            uploads,
            jobs,
        })
    })?);
    let _ = tauri::async_runtime::block_on(core.account.bootstrap());

    #[cfg(debug_assertions)]
    crate::dev::autoconnect(&core);

    let cache = Arc::new(ThumbCache::default_cache(&app_data)?);

    // Uploads a previous process left behind go back in the queue, so they
    // resume from their sessions. Recovering twenty thousand rows is one read
    // and one submission each: the queue runs them on its own budget.
    let _ = tauri::async_runtime::block_on(core.uploads.recover());

    // One bridge for the whole board: the page picks its rows by key instead of
    // listening to an event per kind of work. The first pass fires at once, so a
    // window that loads while jobs are queued is not left with an empty board.
    {
        let mut board = core.jobs.watch();
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let current = board.borrow_and_update().clone();
                let _ = handle.emit("job://changed", current.as_ref());
                if board.changed().await.is_err() {
                    break;
                }
            }
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
