//! Imports shared by the adapter modules.

pub(crate) use std::path::PathBuf;

pub(crate) use tauri::{AppHandle, Emitter, State};

pub(crate) use source_core::app::account::{
    AccountError, AccountState, Login2faParams, LoginOutcome, LoginParams, SetServerParams,
};
pub(crate) use source_core::app::albums::{
    AddAlbumParams, AddLocalAlbumParams, AlbumError, AlbumPageParams, AlbumParams,
    LocalAlbumParams, LocalAlbumSummary, LocalBrowseParams, MoveAssetsParams, SetAlbumCoverParams,
    SetAlbumDescriptionParams,
};
pub(crate) use source_core::app::browse::{
    AssetDetail, AssetDetailParams, BrowseError, ExportAssetsParams, ExportError, ExportResult,
    MediaSourceParams, SearchParams, SetFavoriteParams, TimelinePageParams,
};
pub(crate) use source_core::app::folders::{
    CreateFolderParams, FolderError, ListRemoteFoldersParams, RemoteFolderPage,
};
pub(crate) use source_core::app::sync::SyncError;
pub(crate) use source_core::app::uploads::{
    EnqueueUploadsParams, QueuedUpload, UploadError, UploadTaskParams,
};
pub(crate) use source_core::app::{AppInfo, CatalogChangeKind, CatalogChangedEvent};
pub(crate) use source_core::media::{
    CacheStats, LocalAssetPage, LocalAssetPageParams, LocalBrowsePage, MediaError,
    MediaPermissionState,
};
pub(crate) use source_core::remote::{Capacity, SiteInfo};
pub(crate) use source_core::store::{
    AlbumSummary, AssetSummary, CatalogError, FolderRef, TimelinePage,
};

pub(crate) use source_core::app::Job;

pub(crate) use crate::state::AppState;
