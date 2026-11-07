//! Startup wiring: the composition root and resuming what a crash left behind.

use std::sync::Arc;

use source_core::app::Jobs;
use source_core::app::uploads::{UploadProgressEvent, UploadSinks, UploadStateEvent};
use source_core::app::{CatalogChangeKind, CatalogChangedEvent};
use source_core::store::AssetSummary;
use source_core::{
    AccountService, AlbumsService, BrowseService, CloudreveAuth, CloudreveStorage, CredentialStore,
    FoldersService, SettingsStore, SqliteCatalog, SyncService, ThumbCache, UploadSessionPort,
    UploadsService,
};
use tauri::{Emitter, Manager};

#[cfg(not(debug_assertions))]
use crate::credentials::KeyringCredentialStore;
use crate::state::{App, AppState, JOBS_PAUSED_KEY, LocalMedia};

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

    let core: Arc<App> = Arc::new(
        tauri::async_runtime::block_on(async {
            let catalog = SqliteCatalog::open(app_data.join("catalog.sqlite3")).await?;
            let paused = catalog.get_setting(JOBS_PAUSED_KEY).await?.as_deref() == Some("1");
            let settings: Arc<dyn SettingsStore> = Arc::new(catalog.clone());
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
            // The adapter is the one place that sees a refused session, so it is
            // the one place that reports it; the shell only listens.
            let lost = app.handle().clone();
            auth.on_session_lost(Arc::new(move || {
                let _ = lost.emit("session://lost", ());
            }));
            let cloud = CloudreveStorage::new(Arc::new(auth.clone()), sessions);
            let local = LocalMedia::new();
            let jobs = Jobs::new(JOB_CONCURRENCY);
            if paused {
                jobs.pause();
            }
            let uploads = UploadsService::new(
                cloud.clone(),
                catalog.clone(),
                local.clone(),
                sinks,
                jobs.clone(),
            );
            Ok::<_, source_core::store::CatalogError>(App {
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
                settings,
            })
        })
        .map_err(|cause| std::io::Error::other(format!("{cause:?}")))?,
    );
    let _ = tauri::async_runtime::block_on(core.account.bootstrap());

    #[cfg(debug_assertions)]
    crate::dev::autoconnect(&core);

    let cache = Arc::new(
        ThumbCache::default_cache(&app_data)
            .map_err(|cause| std::io::Error::other(format!("{cause:?}")))?,
    );

    // Uploads a previous process left behind go back in the queue, so they
    // resume from their sessions. Recovery only rewrites rows: the upload
    // service is already ticking and starts them on its own budget.
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

    // Android freezes, then kills, a background process. The settings switch is
    // the one thing that decides whether the process is held: it is the user's
    // choice for background work, restored here before the window loads, and
    // jobs coming and going must not flick it on and off.
    #[cfg(target_os = "android")]
    {
        let hold = !core.jobs.is_paused();
        tauri::async_runtime::spawn(async move {
            crate::android_keep_alive::sync(hold).await;
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
