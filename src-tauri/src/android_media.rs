//! Android media adapter: [`LocalMediaPort`] over MediaStore, through the
//! `MediaStorePlugin` Kotlin bridge.
//!
//! A phone has no home directory and no directory a plain file API may walk, so
//! the desktop model - one directory the user adds - becomes a bucket here: a
//! `RELATIVE_PATH` such as `DCIM/Camera/`, picked out of a flat list, whose ids
//! are `content://` uris. Opening that list is also what raises the system
//! permission dialog; nothing else in the app asks for it.

use std::sync::OnceLock;

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use source_core::media::{
    LocalAsset, LocalAssetPage, LocalBrowsePage, LocalEntry, LocalMediaPort, MediaError,
    MediaPermissionState,
};
use tauri::Wry;
use tauri::plugin::PluginHandle;
use tauri::plugin::mobile::PluginInvokeError;

/// Where the Kotlin class lives: the package it declares, then the class name.
const PACKAGE: &str = "top.k88936.source";
const CLASS: &str = "MediaStorePlugin";

/// The plugin's name on both sides of the bridge; every Kotlin command is
/// reached through it.
const PLUGIN: &str = "mediastore";

static HANDLE: OnceLock<PluginHandle<Wry>> = OnceLock::new();

/// Register the Kotlin bridge with the Android layer.
///
/// The plugin has no webview commands - it exists so Rust can reach the class -
/// so nothing here reaches `tauri.conf.json` or the capability files.
pub(crate) fn plugin() -> tauri::plugin::TauriPlugin<Wry> {
    tauri::plugin::Builder::new(PLUGIN)
        .setup(|_app, api| {
            let handle = api.register_android_plugin(PACKAGE, CLASS)?;
            let _ = HANDLE.set(handle);
            Ok(())
        })
        .build()
}

/// MediaStore, as the local media port.
///
/// The bridge is looked up per call rather than held in the struct: it is
/// registered by the plugin above, which runs after the composition root that
/// constructs this adapter.
#[derive(Clone, Copy)]
pub(crate) struct AndroidMedia;

impl AndroidMedia {
    pub(crate) fn new() -> Self {
        Self
    }

    async fn call<T>(&self, command: &str, payload: impl Serialize) -> Result<T, MediaError>
    where
        T: serde::de::DeserializeOwned,
    {
        let handle = HANDLE.get().ok_or_else(|| MediaError::Platform {
            cause: "MediaStore 插件未注册".to_string(),
        })?;
        handle
            .run_mobile_plugin_async(command, payload)
            .await
            .map_err(rejected)
    }
}

#[async_trait]
impl LocalMediaPort for AndroidMedia {
    async fn get_permission_state(&self) -> Result<MediaPermissionState, MediaError> {
        self.call("permissionState", ()).await
    }

    async fn request_permission(&self) -> Result<MediaPermissionState, MediaError> {
        self.call("requestPermission", ()).await
    }

    /// MediaStore is flat: there is no parent to walk and no current directory,
    /// so the whole library comes back as one list of buckets.
    async fn browse(&self, _parent: Option<&str>) -> Result<LocalBrowsePage, MediaError> {
        let buckets: Buckets = self.call("browse", ()).await?;
        Ok(LocalBrowsePage {
            current: None,
            parent: None,
            entries: buckets.entries,
        })
    }

    async fn list_assets(
        &self,
        album_uri: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, MediaError> {
        self.call(
            "assets",
            AssetPageArgs {
                uri: album_uri,
                cursor,
                limit,
            },
        )
        .await
    }

    async fn get_asset(&self, id: &str) -> Result<Option<LocalAsset>, MediaError> {
        self.call("asset", AssetArgs { id }).await
    }

    async fn count_assets(&self, album_uri: &str) -> Result<u32, MediaError> {
        self.call("count", UriArgs { uri: album_uri }).await
    }

    async fn get_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, MediaError> {
        let encoded: String = self.call("thumb", ThumbArgs { id, edge }).await?;
        STANDARD
            .decode(encoded)
            .map_err(|error| MediaError::Platform {
                cause: format!("无法解码缩略图: {error}"),
            })
    }
}

/// Kotlin answers the picker with the buckets it found, and nothing else: a
/// flat list has no current directory and no "up".
#[derive(Deserialize)]
struct Buckets {
    entries: Vec<LocalEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetPageArgs<'a> {
    uri: &'a str,
    cursor: Option<&'a str>,
    limit: u32,
}

#[derive(Serialize)]
struct UriArgs<'a> {
    uri: &'a str,
}

#[derive(Serialize)]
struct AssetArgs<'a> {
    id: &'a str,
}

#[derive(Serialize)]
struct ThumbArgs<'a> {
    id: &'a str,
    edge: u32,
}

/// A rejection carries the message the user should see, and a permission one
/// is a different situation from a query that blew up, so its code survives.
fn rejected(error: PluginInvokeError) -> MediaError {
    match error {
        PluginInvokeError::InvokeRejected(response) => {
            let message = response
                .message
                .unwrap_or_else(|| "无法访问系统相册".to_string());
            match response.code.as_deref() {
                Some("permission" | "denied") => MediaError::PermissionDenied,
                _ => MediaError::Platform { cause: message },
            }
        }
        other => MediaError::Platform {
            cause: format!("无法访问系统相册: {other}"),
        },
    }
}
