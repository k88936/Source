//! Shared Tauri state.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use source_core::app::Jobs;
use source_core::{
    AccountService, AlbumsService, BrowseService, CloudreveAuth, CloudreveStorage, FoldersService,
    SettingsStore, SqliteCatalog, SyncService, ThumbCache, UploadsService,
};
use source_subscription::Change;

/// Where the settings page keeps whether background work is paused.
pub(crate) const JOBS_PAUSED_KEY: &str = "jobs.paused";

/// Where the settings page keeps whether the platform should hold the process
/// in the background. Only Android acts on it.
pub(crate) const KEEP_ALIVE_KEY: &str = "keep.alive";

/// Where local albums come from on this platform: the filesystem on desktop,
/// MediaStore on a phone.
#[cfg(target_os = "android")]
pub(crate) type LocalMedia = crate::android_media::AndroidMedia;
#[cfg(not(target_os = "android"))]
pub(crate) type LocalMedia = source_core::FsLocalMedia;

/// The composed dependency graph.
///
/// These aliases are the wiring table: swapping an adapter is changing a type
/// argument here, and the compiler checks every use case against it. Only
/// `bootstrap::setup` builds these, and it shares one Cloudreve auth handle so a
/// login is visible to every service at once.
pub(crate) type Account = AccountService<CloudreveAuth, CloudreveStorage, SqliteCatalog>;
pub(crate) type Folders = FoldersService<CloudreveStorage, SqliteCatalog>;
pub(crate) type Albums = AlbumsService<CloudreveStorage, SqliteCatalog, LocalMedia>;
pub(crate) type Browse = BrowseService<CloudreveStorage, SqliteCatalog, LocalMedia>;
pub(crate) type Uploads = UploadsService<CloudreveStorage, SqliteCatalog, LocalMedia>;
pub(crate) type Sync = SyncService<CloudreveStorage, SqliteCatalog, LocalMedia>;

pub(crate) struct App {
    pub(crate) account: Arc<Account>,
    pub(crate) folders: Arc<Folders>,
    pub(crate) albums: Arc<Albums>,
    pub(crate) browse: Arc<Browse>,
    pub(crate) uploads: Arc<Uploads>,
    pub(crate) sync: Arc<Sync>,
    /// The queue every background job goes through, and the board the UI reads.
    pub(crate) jobs: Jobs,
    pub(crate) settings: Arc<dyn SettingsStore>,
    /// Whether the stored session is still being checked. The restore loop is
    /// the only writer; `session_state` reads it to say what the page is
    /// waiting for.
    pub(crate) restoring: AtomicBool,
}

impl App {
    /// One commit, answered by every view a page may be watching.
    ///
    /// Each service owns the manager of its own view, so the list of views is
    /// the list of services and a snapshot in one of them cannot hold up the
    /// deltas of another. The commit was already observed as committed, so this
    /// is never called for a rolled-back transaction.
    pub(crate) async fn apply(&self, changes: &[Change]) {
        tokio::join!(
            self.albums.apply_changes(changes),
            self.browse.apply_changes(changes),
        );
    }
}

/// Everything a command needs: the use cases, the thumbnail cache, and through
/// the services the subscriptions the page holds.
pub(crate) struct AppState {
    pub(crate) core: Arc<App>,
    pub(crate) cache: Arc<ThumbCache>,
    pub(crate) app_data: PathBuf,
}
