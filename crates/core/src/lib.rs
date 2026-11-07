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
pub mod exif;
pub mod mapping;
pub mod media;
pub mod remote;
pub mod sqlite;
pub mod store;
pub mod task;
pub mod time;

pub use app::{
    AccountService, AccountState, AlbumsService, BrowseService, FoldersService, LoginOutcome,
    SyncService, UploadOutcome, UploadsService,
};
pub use credential::CredentialStore;
pub use media::{FsLocalMedia, LOCAL_THUMB_EDGE, ThumbCache, preview_from_jpeg};
pub use remote::{
    CapacityError, CloudreveAuth, CloudreveStorage, CreateFolderError, CurrentAccount, ListError,
    MediaFetchError, MediaRange, MetadataError, MoveError, ProbeError, ProgressSink, RemoteAuth,
    RemoteEntry, RemoteEntryPage, RemoteError, RemoteStorage, SessionError, SignInError,
    SignInOutcome, SignedIn, SiteInfo, UploadProgress, UploadRequest, UploadUriError,
};
pub use sqlite::SqliteCatalog;
pub use store::{
    Account, AccountStore, AlbumStore, AssetRecord, AssetStore, FolderRecord, FolderStore,
    FolderSyncState, LocalStore, SearchQuery, SettingsStore, TimelineStore, UploadQueueStore,
    UploadSessionPort, UserInfo,
};
