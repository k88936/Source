//! Startup wiring: the composition root and resuming what a crash left behind.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use source_core::app::Jobs;
use source_core::app::account::{RestoreOutcome, SessionSetup, SessionState};
use source_core::app::jobs::JobList;
use source_core::app::uploads::{UploadSinks, UploadStateEvent};
use source_core::app::{Schedule, SettingsService};
use source_core::store::observe_commits;
use source_core::{
    AccountService, AlbumsService, BrowseService, CloudreveAuth, CloudreveStorage, CredentialStore,
    FoldersService, SettingsStore, SqliteCatalog, SyncService, ThumbCache, UploadSessionPort,
    UploadsService,
};
use source_subscription::SubscriptionManager;
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

    // Commit batches are observed from a process-wide hook, so the channel is
    // made before any connection is opened.
    let (changes, mut observed) = tokio::sync::mpsc::unbounded_channel();
    let core = tauri::async_runtime::block_on(async {
        // The observer goes in before the first connection is opened: SQLite
        // only applies an auto-extension to connections opened after it is
        // registered.
        observe_commits(changes);
        let catalog = SqliteCatalog::open(app_data.join("catalog.sqlite3")).await?;
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
        let uploads = UploadsService::new(
            cloud.clone(),
            catalog.clone(),
            local.clone(),
            sinks,
            jobs.clone(),
        );
        let job_list = Arc::new(SubscriptionManager::new(Arc::new(JobList::new(
            jobs.clone(),
        ))));
        let running = uploads.running();
        Ok::<_, source_core::store::CatalogError>(Arc::new(App {
            account: Arc::new(AccountService::new(auth, cloud.clone(), catalog.clone())),
            folders: Arc::new(FoldersService::new(cloud.clone(), catalog.clone())),
            albums: Arc::new(AlbumsService::new(
                cloud.clone(),
                catalog.clone(),
                local.clone(),
                jobs.clone(),
                Schedule::new(Arc::clone(&settings)),
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
                Schedule::new(Arc::clone(&settings)),
            )),
            uploads,
            jobs,
            job_list,
            settings: SettingsService::new(settings),
            running,
            restoring: AtomicBool::new(true),
        }))
    })
    .map_err(|cause| std::io::Error::other(format!("{cause:?}")))?;
    // Commits arrive in the order they were committed, so the loop answers them
    // in order and the page needs no revision number. It starts after the graph
    // exists, because answering one means reaching every view through its
    // service.
    {
        let applying = Arc::clone(&core);
        tauri::async_runtime::spawn(async move {
            while let Some(batch) = observed.recv().await {
                applying.apply(&batch).await;
            }
        });
    }
    // The queue's own view is fed by the queue: every update names the rows it
    // touched, and the view asks them back. There is no snapshot to emit here,
    // because a page that connects gets one from the manager.
    {
        let mut changes = core.jobs.observe();
        let jobs = Arc::clone(&core.job_list);
        tauri::async_runtime::spawn(async move {
            while let Some(batch) = changes.recv().await {
                jobs.apply(batch.as_ref()).await;
            }
        });
    }
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

    // The two periodic passes each watch their own frequency. They read the
    // same setting and record their own clock, so turning one on does not make
    // the other run. They are spawned here because they need this process's
    // executor, which core does not own.
    tauri::async_runtime::spawn(core.sync.start_schedule());
    tauri::async_runtime::spawn(core.albums.start_schedule());

    // Android freezes, then kills, a background process. The settings switch is
    // the one thing that decides whether the process is held: it is the user's
    // choice for background work, restored here before the window loads.
    #[cfg(target_os = "android")]
    {
        let hold = tauri::async_runtime::block_on(core.settings.get())
            .map_err(|cause| std::io::Error::other(format!("{cause:?}")))?
            .keep_alive;
        tauri::async_runtime::spawn(async move {
            tauri_plugin_keep_alive::sync(hold).await;
        });
    }

    let handle = app.handle().clone();
    app.manage(AppState {
        core: Arc::clone(&core),
        cache,
        app_data,
    });
    // The stored session is checked off this thread, once the window has
    // everything it needs to open on the row in sqlite.
    restore_session(handle, core);
    Ok(())
}

/// How long a server that answered nothing is left alone before it is asked
/// again.
const RESTORE_RETRY: Duration = Duration::from_secs(10);

/// Check the stored session in the background, then say how it went.
///
/// A start never waits for a server: the window opens from the account row in
/// sqlite, and this task reports its verdict as `session://setup`, or as
/// `session://lost` when the server refused the stored session.
///
/// A server that answered nothing is asked again. A port forward in front of a
/// container that is still coming up accepts the connection and says nothing,
/// which is a moment in its start rather than a dead address; the row on disk
/// is kept either way, so waiting costs nothing.
fn restore_session(handle: tauri::AppHandle, core: Arc<App>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let outcome = core.account.restore().await;
            core.restoring
                .store(false, std::sync::atomic::Ordering::Relaxed);
            match outcome {
                Ok(RestoreOutcome::Empty) => return,
                Ok(RestoreOutcome::Restored { account }) => {
                    let _ = handle.emit(
                        "session://setup",
                        SessionState {
                            account: Some(account),
                            setup: SessionSetup::Live,
                        },
                    );
                    return;
                }
                Ok(RestoreOutcome::Rejected) => {
                    let _ = handle.emit("session://lost", ());
                    return;
                }
                Ok(RestoreOutcome::Unreachable { account }) => {
                    let _ = handle.emit(
                        "session://setup",
                        SessionState {
                            account,
                            setup: SessionSetup::Offline,
                        },
                    );
                }
                Err(cause) => {
                    // The catalog refused a write, which a retry cannot mend.
                    eprintln!("session: restore failed: {cause:?}");
                    return;
                }
            }
            tokio::time::sleep(RESTORE_RETRY).await;
        }
    });
}

/// Where the upload queue reports: the webview's events.
fn upload_sinks(handle: &tauri::AppHandle) -> UploadSinks {
    let handle = handle.clone();
    UploadSinks {
        state: Arc::new(move |event: &UploadStateEvent| {
            let _ = handle.emit("upload://state", event);
        }),
    }
}
