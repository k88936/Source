//! Shared Tauri state.

use std::path::PathBuf;
use std::sync::Arc;

use source_core::{
    BrowseService, CloudreveService, FoldersService, FsLocalMedia, IndexService, SessionService,
    SqliteCatalog, ThumbCache, UploadsService,
};

/// The composed dependency graph.
///
/// These aliases are the wiring table: swapping an adapter is changing a type
/// argument here, and the compiler checks every use case against it. Only
/// `bootstrap::setup` builds these, and it shares one Cloudreve handle so a
/// login is visible to every service at once.
pub(crate) type Session = SessionService<CloudreveService, SqliteCatalog>;
pub(crate) type Index = IndexService<CloudreveService, SqliteCatalog>;
pub(crate) type Folders = FoldersService<CloudreveService, SqliteCatalog>;
pub(crate) type Browse = BrowseService<CloudreveService, SqliteCatalog, FsLocalMedia>;
pub(crate) type Uploads = UploadsService<CloudreveService, SqliteCatalog, FsLocalMedia>;

pub(crate) struct App {
    pub(crate) session: Arc<Session>,
    pub(crate) index: Arc<Index>,
    pub(crate) folders: Arc<Folders>,
    pub(crate) browse: Arc<Browse>,
    pub(crate) uploads: Arc<Uploads>,
}

/// Everything a command needs: the use cases plus the thumbnail cache.
pub(crate) struct AppState {
    pub(crate) core: Arc<App>,
    pub(crate) cache: Arc<ThumbCache>,
    pub(crate) app_data: PathBuf,
}
