//! Android media adapter: [`LocalMediaPort`] over the `mediastore` plugin.
//!
//! The plugin speaks MediaStore's own language - buckets, `content://` uris,
//! camelCase rows - and knows nothing of this app's model. This is the thin
//! seam that turns each answer into [`LocalEntry`] / [`LocalAsset`] and each
//! failure into a [`MediaError`]; there is no other logic here.
//!
//! A phone has no home directory and no directory a plain file API may walk, so
//! the desktop model - one directory the user adds - becomes a bucket here: a
//! `RELATIVE_PATH` such as `DCIM/Camera/`, picked out of a flat list, whose ids
//! are `content://` uris.

use async_trait::async_trait;
use source_core::media::{
    LocalAsset, LocalAssetPage, LocalBrowsePage, LocalEntry, LocalMediaPort, MediaError,
    MediaPermissionState,
};
use source_core::store::LocalAssetId;
use tauri_plugin_mediastore as mediastore;
use tauri_plugin_mediastore::{Asset, Error, Grant};

/// MediaStore, as the local media port.
#[derive(Clone, Copy)]
pub(crate) struct AndroidMedia;

impl AndroidMedia {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LocalMediaPort for AndroidMedia {
    async fn get_permission_state(&self) -> Result<MediaPermissionState, MediaError> {
        let state = mediastore::permission_state().await.map_err(media_error)?;
        Ok(permission(state))
    }

    async fn request_permission(&self) -> Result<MediaPermissionState, MediaError> {
        let state = mediastore::request_permission()
            .await
            .map_err(media_error)?;
        Ok(permission(state))
    }

    /// MediaStore is flat: there is no parent to walk and no current directory,
    /// so the whole library comes back as one list of buckets.
    async fn browse(&self, _parent: Option<&str>) -> Result<LocalBrowsePage, MediaError> {
        let entries = mediastore::browse()
            .await
            .map_err(media_error)?
            .into_iter()
            .map(entry)
            .collect();
        Ok(LocalBrowsePage {
            current: None,
            parent: None,
            entries,
        })
    }

    async fn list_assets(
        &self,
        album_uri: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, MediaError> {
        let page = mediastore::assets(album_uri, cursor, limit)
            .await
            .map_err(media_error)?;
        Ok(LocalAssetPage {
            items: page.items.into_iter().map(asset).collect(),
            next: page.next,
        })
    }

    async fn get_asset(&self, id: &str) -> Result<Option<LocalAsset>, MediaError> {
        Ok(mediastore::asset(id).await.map_err(media_error)?.map(asset))
    }

    async fn count_assets(&self, album_uri: &str) -> Result<u32, MediaError> {
        Ok(mediastore::count(album_uri).await.map_err(media_error)?)
    }

    async fn get_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, MediaError> {
        Ok(mediastore::thumb(id, edge).await.map_err(media_error)?)
    }
}

fn permission(state: mediastore::PermissionState) -> MediaPermissionState {
    MediaPermissionState {
        granted: match state.granted {
            Grant::Full => source_core::media::PermissionGrant::Full,
            Grant::Partial => source_core::media::PermissionGrant::Partial,
            Grant::Denied => source_core::media::PermissionGrant::Denied,
        },
        can_request: state.can_request,
    }
}

fn entry(entry: mediastore::Entry) -> LocalEntry {
    LocalEntry {
        uri: entry.uri,
        name: entry.name,
        path: entry.path,
    }
}

fn asset(asset: Asset) -> LocalAsset {
    LocalAsset {
        id: LocalAssetId(asset.id),
        // MediaStore rows carry no album of their own: the bucket the user
        // added is the album, and it lives outside this row.
        album_id: None,
        name: asset.name,
        mime: asset.mime,
        size: asset.size,
        date_taken: asset.date_taken,
        uri: asset.uri,
        width: asset.width,
        height: asset.height,
        duration_ms: asset.duration_ms,
    }
}

/// The plugin's failures are the platform's, but the port speaks one error: a
/// denial is a situation the user can still act on, everything else is a query
/// that did not come back.
fn media_error(error: Error) -> MediaError {
    match error {
        Error::PermissionDenied => MediaError::PermissionDenied,
        other => MediaError::Platform {
            cause: other.to_string(),
        },
    }
}
