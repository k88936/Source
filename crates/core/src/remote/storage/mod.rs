//! Every file operation against a Cloudreve server: the contract the use cases
//! call, and the adapter that implements it.
//!
//! The adapter reaches the signed-in client through [`CurrentAccount`], so the
//! storage half can depend on auth without being able to touch credentials.

use std::sync::Arc;

use async_trait::async_trait;
use cloudreve_sdk_api::Client;

use crate::error::AppError;
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
    /// The account's quota, read from the server rather than cached.
    async fn capacity(&self) -> Result<Capacity, AppError>;

    /// One page of `uri`'s children, with `uri` itself as `current`.
    async fn list_entries(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteEntryPage, AppError>;

    /// The folder `uri` itself, the drive root included.
    async fn folder_info(&self, uri: &str) -> Result<RemoteEntry, AppError>;

    /// Thumbnail bytes for `uri`, signed and fetched in one step so the URL
    /// never leaves the adapter.
    async fn thumb_bytes(&self, uri: &str) -> Result<Vec<u8>, AppError>;

    /// Whole original file for `uri`.
    async fn media_bytes(&self, uri: &str) -> Result<Vec<u8>, AppError>;

    /// A `Range` request against the original file, answered as the server
    /// sent it. `range=None` fetches the whole file.
    async fn media_range(&self, uri: &str, range: Option<&str>) -> Result<MediaRange, AppError>;

    /// Signed URL of the original file, for playing it straight from the
    /// server: WebKitGTK's media player rejects custom schemes, so desktop
    /// cannot stream `<video>` through the `media` protocol.
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
