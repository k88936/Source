//! The whole engine: domain types, use cases, and the adapters that connect
//! them to SQLite, Cloudreve and the platform media library.
//!
//! Each module holds one dependency direction end to end - its contract, the
//! payload types of that contract, and the code that implements it. Everything
//! here is independent of Tauri: the Tauri crate only wires these pieces
//! together and maps commands. TypeScript bindings are generated from those
//! payload types, wherever they are declared.

pub mod app;
pub mod credential;
pub mod error;
pub mod exif;
pub mod mapping;
pub mod media;
pub mod remote;
pub mod sqlite;
pub mod store;
pub mod time;

pub use app::{
    AccountService, AccountState, AlbumsService, BrowseService, FoldersService, LoginOutcome,
    UploadOutcome, UploadsService,
};
pub use credential::CredentialStore;
pub use error::{AppError, ErrorCode};
pub use media::{FsLocalMedia, LOCAL_THUMB_EDGE, ThumbCache, preview_from_jpeg};
pub use remote::{
    CloudreveAuth, CloudreveStorage, CurrentAccount, MediaRange, ProgressSink, RemoteAuth,
    RemoteEntry, RemoteEntryPage, RemoteStorage, SignInOutcome, SignedIn, SiteInfo, UploadProgress,
    UploadRequest,
};
pub use sqlite::SqliteCatalog;
pub use store::{
    Account, AccountStore, AlbumStore, AssetRecord, AssetStore, FolderRecord, FolderStore,
    FolderSyncState, LocalStore, SearchQuery, SettingsStore, TimelineStore, UploadQueueStore,
    UploadSessionPort, UserInfo,
};
