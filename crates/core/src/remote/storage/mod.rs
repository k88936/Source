//! Every file operation against a Cloudreve server: the contract the use cases
//! call, and the adapter that implements it.
//!
//! The adapter reaches the signed-in client through [`CurrentAccount`], so the
//! storage half can depend on auth without being able to touch credentials.

use async_trait::async_trait;

use crate::remote::{
    CapacityError, CreateFolderError, ListError, MediaFetchError, MetadataError, MoveError,
    UploadUriError,
};
use crate::remote::auth::CurrentAccount;

mod cloudreve;
mod dto;
mod entries;
mod upload;

pub use cloudreve::CloudreveStorage;
pub use dto::{
    Capacity, MediaRange, ProgressSink, RemoteEntry, RemoteEntryPage, UploadProgress, UploadRequest,
};
pub(crate) use entries::*;

pub(crate) const PAGE_SIZE: i32 = 100;

#[async_trait]
pub trait RemoteStorage: Send + Sync {
    /// Whether the adapter has a signed-in client behind it right now.
    ///
    /// The upload queue asks before it starts a row: without a session every
    /// attempt fails as an expired one and the backlog loses its place.
    fn is_live(&self) -> bool;

    /// The account's quota, read from the server rather than cached.
    async fn get_capacity(&self) -> Result<Capacity, CapacityError>;

    /// One page of `uri`'s children, with `uri` itself as `current`.
    async fn list_entries(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteEntryPage, ListError>;

    /// The folder `uri` itself, the drive root included.
    async fn get_folder_info(&self, uri: &str) -> Result<RemoteEntry, ListError>;

    /// Thumbnail bytes for `uri`, signed and fetched in one step so the URL
    /// never leaves the adapter.
    async fn get_thumb_bytes(&self, uri: &str) -> Result<Vec<u8>, MediaFetchError>;

    /// Whole original file for `uri`.
    async fn get_media_bytes(&self, uri: &str) -> Result<Vec<u8>, MediaFetchError>;

    /// A `Range` request against the original file, answered as the server
    /// sent it. `range=None` fetches the whole file.
    async fn get_media_range(
        &self,
        uri: &str,
        range: Option<&str>,
    ) -> Result<MediaRange, MediaFetchError>;

    /// Signed URL of the original file, for playing it straight from the
    /// server: WebKitGTK's media player rejects custom schemes, so desktop
    /// cannot stream `<video>` through the `media` protocol.
    async fn get_media_url(&self, uri: &str) -> Result<String, MediaFetchError>;

    /// Download just the head of a remote file and parse its EXIF timestamp.
    /// Used only when Cloudreve did not extract `exif:taken_at` server-side.
    async fn client_taken_at(&self, uri: &str) -> Result<Option<i64>, MediaFetchError>;

    /// Patch key/value metadata. `None` removes the key.
    async fn patch_metadata(
        &self,
        uri: &str,
        patches: &[(String, Option<String>)],
    ) -> Result<(), MetadataError>;

    async fn create_folder(
        &self,
        parent_uri: &str,
        name: &str,
    ) -> Result<RemoteEntry, CreateFolderError>;

    /// Move files into `dest_uri`, returning their new URIs in input order.
    ///
    /// In Cloudreve an album is a folder, so "add to album" is a move rather
    /// than a copy, and the URI (which embeds the path) changes with it.
    async fn move_assets(&self, uris: &[String], dest_uri: &str) -> Result<Vec<String>, MoveError>;

    /// Upload a local asset. `source_uri` is `file://...` on desktop or
    /// `content://...` on Android; adapters must stream it without copying to
    /// an app cache and must stop between chunks once `cancel` is raised.
    async fn upload_uri(&self, request: UploadRequest) -> Result<RemoteEntry, UploadUriError>;
}
