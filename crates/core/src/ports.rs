//! Ports implemented by the adapter crates. `core` owns the use cases; it never
//! talks to Cloudreve, SQLite or the filesystem directly.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use async_trait::async_trait;

use crate::dto::{
    AlbumSummary, AppError, AssetSummary, Capacity, FolderRef, IndexPhase, LocalAlbum, LocalAsset,
    LocalAssetPage, LoginOutcome, MediaPermissionState, SessionState, SiteInfo, TakenAtSource,
    TimelineCursor, TimelinePage, UploadTask, UploadTaskId,
};

pub type ProgressSink = Arc<dyn Fn(UploadProgress) + Send + Sync>;

#[derive(Debug, Clone, Copy, Default)]
pub struct UploadProgress {
    pub uploaded: i64,
    pub total: i64,
    pub speed: i64,
    pub eta_seconds: Option<u32>,
}

/// One upload attempt: what to send, where, and how to report it.
///
/// `task_id` doubles as the key of the resumable journal, so a retry of the
/// same queue entry finds the chunks an earlier attempt already finished.
pub struct UploadRequest {
    pub task_id: UploadTaskId,
    pub source_uri: String,
    pub size: i64,
    pub dest_uri: String,
    pub name: String,
    pub mime: Option<String>,
    /// Raised by `cancel_upload`; uploaders stop between two chunks.
    pub cancel: Arc<AtomicBool>,
    pub progress: ProgressSink,
}

/// A file or folder returned by Cloudreve's navigator.
#[derive(Debug, Clone)]
pub struct RemoteEntry {
    pub id: String,
    pub uri: String,
    pub name: String,
    pub path: String,
    pub is_folder: bool,
    pub size: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RemoteEntryPage {
    pub entries: Vec<RemoteEntry>,
    pub next: Option<String>,
    pub parent: Option<RemoteEntry>,
}

#[async_trait]
pub trait CloudrevePort: Send + Sync {
    async fn site_info(&self, base_url: &str) -> Result<SiteInfo, AppError>;

    async fn login(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginOutcome, AppError>;

    async fn login_2fa(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SessionState, AppError>;

    /// Restore a persisted session, validating it against `/user/me`.
    async fn restore(&self, base_url: &str) -> Result<Option<SessionState>, AppError>;

    async fn logout(&self, base_url: &str) -> Result<(), AppError>;

    async fn capacity(&self) -> Result<Capacity, AppError>;

    async fn list_entries(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteEntryPage, AppError>;

    async fn folder_info(&self, uri: &str) -> Result<RemoteEntry, AppError>;

    /// Signed (short-lived) URL for a Cloudreve thumbnail.
    async fn thumb_url(&self, uri: &str) -> Result<String, AppError>;

    /// Signed (short-lived) URL for the original media/stream.
    async fn media_url(&self, uri: &str) -> Result<String, AppError>;

    /// Download just the head of a remote file and parse its EXIF timestamp.
    /// Used only when Cloudreve did not extract `exif:taken_at` server-side.
    async fn client_taken_at(&self, uri: &str) -> Result<Option<i64>, AppError>;

    /// Patch key/value metadata. `None` removes the key.
    async fn patch_metadata(
        &self,
        uri: &str,
        patches: &[(String, Option<String>)],
    ) -> Result<(), AppError>;

    async fn create_folder(&self, parent_uri: &str, name: &str) -> Result<RemoteEntry, AppError>;

    /// Move files into `dest_uri`, returning their new URIs in input order.
    ///
    /// In Cloudreve an album is a folder, so "add to album" is a move rather
    /// than a copy, and the URI (which embeds the path) changes with it.
    async fn move_assets(&self, uris: &[String], dest_uri: &str) -> Result<Vec<String>, AppError>;

    /// Upload a local asset. `source_uri` is `file://...` on desktop or
    /// `content://...` on Android; adapters must stream it without copying to
    /// an app cache and must stop between chunks once `cancel` is raised.
    async fn upload_uri(&self, request: UploadRequest) -> Result<RemoteEntry, AppError>;
}

// ---------------------------------------------------------------------------
// Catalog, split by use case so each module's bounds name what it may touch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StoredServer {
    pub base_url: String,
    pub user_id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderRecord {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub uri: String,
    pub path: String,
    pub updated_at: i64,
    pub is_album: bool,
    pub cover_asset_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderSyncState {
    pub folder_id: String,
    pub status: String,
    pub remote_updated_at: Option<i64>,
    pub last_listed_at: Option<i64>,
    pub last_error: Option<String>,
}

impl FolderSyncState {
    pub fn is_done(&self) -> bool {
        self.status == "done"
    }
}

#[derive(Debug, Clone)]
pub struct AssetRecord {
    pub summary: AssetSummary,
    /// Cloudreve file URI, needed by the thumbnail/media proxies.
    pub uri: String,
    /// Raw `exif:*` metadata as JSON, for the detail sheet.
    pub exif_json: Option<String>,
    pub taken_at_source: TakenAtSource,
    pub remote_updated_at: i64,
}

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub favorite: Option<bool>,
    pub media_type: Option<crate::dto::MediaType>,
    pub cursor: Option<TimelineCursor>,
    pub limit: u32,
}

/// Header of one persisted upload session, written once per session.
///
/// The catalog treats the payload as opaque bytes: only the uploader adapter
/// knows how to encode or interpret it. Expiry is not enforced by the catalog;
/// the uploader reports a session the server no longer knows.
#[derive(Debug, Clone)]
pub struct StoredSession {
    pub bytes: Vec<u8>,
    /// Header format version, so headers from another format are dropped.
    pub version: u32,
    /// Session expiry as a Unix timestamp in seconds.
    pub expires_at: i64,
}

/// One finished chunk of a stored session.
#[derive(Debug, Clone)]
pub struct StoredRecord {
    /// Chunk index; recording the same index again replaces the row.
    pub index: u32,
    pub bytes: Vec<u8>,
}

/// A session header together with the chunks that already finished.
#[derive(Debug, Clone)]
pub struct StoredJournal {
    pub session: StoredSession,
    pub records: Vec<StoredRecord>,
}

/// Durable storage for resumable upload journals.
///
/// A journal is append-only: the header is written once, then one row per
/// finished chunk. Re-appending a chunk index must replace that row instead of
/// growing the journal, so a retried chunk cannot corrupt the replay.
#[async_trait]
pub trait UploadSessionPort: Send + Sync {
    /// Store the header of a new session under `key`, dropping the chunk rows
    /// of the session it replaces.
    async fn create_session(&self, key: &str, session: &StoredSession) -> Result<(), AppError>;
    async fn append_chunk(&self, key: &str, index: u32, record: &[u8]) -> Result<(), AppError>;
    async fn load_session(&self, key: &str) -> Result<Option<StoredJournal>, AppError>;
    /// Remove the header and every chunk recorded under `key`.
    async fn remove_session(&self, key: &str) -> Result<(), AppError>;
}

/// The selected server and the key/value settings stored next to it.
#[async_trait]
pub trait ServerStore: Send + Sync {
    async fn save_server(&self, server: &StoredServer) -> Result<(), AppError>;
    async fn load_server(&self) -> Result<Option<StoredServer>, AppError>;
    async fn clear_server(&self) -> Result<(), AppError>;

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError>;
    async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError>;
}

/// The library root and the remote folders mirrored from the server.
#[async_trait]
pub trait FolderStore: Send + Sync {
    async fn set_library_root(&self, root: &FolderRef) -> Result<(), AppError>;
    async fn library_root(&self) -> Result<Option<FolderRef>, AppError>;

    async fn upsert_folder(&self, folder: &FolderRecord) -> Result<(), AppError>;
    async fn folder(&self, id: &str) -> Result<Option<FolderRecord>, AppError>;
    async fn folder_by_uri(&self, uri: &str) -> Result<Option<FolderRecord>, AppError>;
    async fn folder_state(&self, id: &str) -> Result<Option<FolderSyncState>, AppError>;
    async fn mark_folder(
        &self,
        id: &str,
        status: &str,
        error: Option<&str>,
        remote_updated_at: Option<i64>,
        now: i64,
    ) -> Result<(), AppError>;
    async fn replace_folder_assets(
        &self,
        folder_id: &str,
        assets: &[AssetRecord],
        now: i64,
    ) -> Result<(), AppError>;
    /// Drop every folder outside `keep`, with its assets and sync state.
    ///
    /// Only a completed full pass has listed the whole tree, so a caller must
    /// not prune after a partial or incremental run.
    async fn retain_folders(&self, keep: &HashSet<String>) -> Result<(), AppError>;
}

/// Indexed assets, local assets and the per-asset flags the UI can toggle.
#[async_trait]
pub trait AssetStore: Send + Sync {
    async fn upsert_asset(&self, asset: &AssetRecord) -> Result<(), AppError>;
    /// Re-point an asset at another folder and URI after a remote move.
    async fn move_asset(&self, asset_id: &str, folder_id: &str, uri: &str) -> Result<(), AppError>;
    async fn asset(&self, id: &str) -> Result<Option<AssetRecord>, AppError>;
    async fn set_favorite(&self, asset_id: &str, value: bool) -> Result<(), AppError>;
    /// Store the blur-up mosaic for a tile. Written once per asset, lazily.
    async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), AppError>;

    async fn save_local_asset(&self, asset: &LocalAsset) -> Result<(), AppError>;
    async fn local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError>;
}

/// Albums are folders with media in them; the counts and covers are derived.
#[async_trait]
pub trait AlbumStore: Send + Sync {
    async fn mark_album(&self, folder_id: &str, is_album: bool) -> Result<(), AppError>;
    async fn rebuild_albums(&self) -> Result<(), AppError>;
    async fn album_list(&self) -> Result<Vec<AlbumSummary>, AppError>;
    async fn album(&self, id: &str) -> Result<Option<AlbumSummary>, AppError>;
    async fn album_page(
        &self,
        album_id: &str,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError>;
    async fn set_album_cover(&self, album_id: &str, cover: Option<&str>) -> Result<(), AppError>;
    async fn set_album_description(
        &self,
        album_id: &str,
        description: Option<&str>,
    ) -> Result<(), AppError>;
}

/// Read paths over the whole index: the timeline and search.
#[async_trait]
pub trait TimelineStore: Send + Sync {
    async fn timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError>;
    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, AppError>;
}

/// The upload queue persisted across restarts.
#[async_trait]
pub trait UploadQueueStore: Send + Sync {
    async fn enqueue_upload(&self, task: &UploadTask) -> Result<(), AppError>;
    async fn update_upload(&self, task: &UploadTask) -> Result<(), AppError>;
    async fn upload_task(&self, id: &str) -> Result<Option<UploadTask>, AppError>;
    async fn upload_tasks(&self) -> Result<Vec<UploadTask>, AppError>;
    async fn pending_uploads(&self) -> Result<Vec<UploadTask>, AppError>;
}

// ---------------------------------------------------------------------------
// Local media (MediaStore on Android, filesystem on desktop dev builds)
// ---------------------------------------------------------------------------

pub trait LocalMediaPort: Send + Sync {
    fn permission_state(&self) -> MediaPermissionState;
    fn request_permission(&self) -> MediaPermissionState;
    fn albums(&self) -> Result<Vec<LocalAlbum>, AppError>;
    fn assets(
        &self,
        album_id: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, AppError>;
    fn asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError>;

    /// Small JPEG preview for a grid tile.
    ///
    /// Each platform picks its cheapest source: MediaStore thumbnails on
    /// Android, decode or `ffmpeg` on desktop.
    fn thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, AppError>;
}

// ---------------------------------------------------------------------------
// Index progress
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IndexProgress {
    pub phase: IndexPhase,
    pub folders_done: u32,
    pub folders_total: u32,
    pub assets_seen: i64,
    pub started_at: Option<i64>,
    pub error: Option<AppError>,
}
