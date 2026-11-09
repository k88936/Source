//! Kotlin bridge to the Android MediaStore photo library.
//!
//! Android has no home directory and no directory a plain file API may walk, so
//! the only way to the device photos is MediaStore rows behind a runtime
//! permission. The platform's model is a flat list of buckets: a `RELATIVE_PATH`
//! such as `DCIM/Camera/` whose ids are `content://` uris. Opening that list is
//! also what raises the system permission dialog; nothing else asks for it.
//!
//! This crate speaks the platform's language only - strings, numbers, camelCase
//! JSON - and knows nothing of the app's domain types, so it can be reused by
//! any Android Tauri app. Mapping into an application's own model is the
//! caller's job.

#[cfg(target_os = "android")]
mod android {
    use std::sync::OnceLock;

    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use tauri::Wry;
    use tauri::plugin::PluginHandle;
    use tauri::plugin::mobile::PluginInvokeError;

    /// Where the Kotlin class lives: the package it declares, then the class name.
    const PACKAGE: &str = "top.k88936.source.mobile.mediastore";
    const CLASS: &str = "MediaStorePlugin";

    /// The plugin's name on both sides of the bridge; every Kotlin command is
    /// reached through it.
    const PLUGIN: &str = "mediastore";

    static HANDLE: OnceLock<PluginHandle<Wry>> = OnceLock::new();

    /// Register the Kotlin bridge with the Android layer.
    ///
    /// The plugin has no webview commands - it exists so Rust can reach the class -
    /// so nothing here reaches `tauri.conf.json` or the capability files.
    pub fn init() -> tauri::plugin::TauriPlugin<Wry> {
        tauri::plugin::Builder::new(PLUGIN)
            .setup(|_app, api| {
                let handle = api.register_android_plugin(PACKAGE, CLASS)?;
                let _ = HANDLE.set(handle);
                Ok(())
            })
            .build()
    }

    /// How much of the device library the app may read.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Grant {
        Full,
        /// Only the files the user picked out of the library (Android 14+).
        Partial,
        Denied,
    }

    /// What the platform allows right now, and whether the user can still be asked.
    #[derive(Debug, Clone, Copy, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PermissionState {
        pub granted: Grant,
        pub can_request: bool,
    }

    /// One bucket: a directory of the library the user can add as an album.
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Entry {
        /// `mediastore:` plus the bucket's `RELATIVE_PATH`; the caller stores it.
        pub uri: String,
        pub name: String,
        /// Where the bucket sits, without the trailing slash.
        pub path: Option<String>,
    }

    /// One row of MediaStore: a photo or a video.
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Asset {
        pub id: String,
        pub uri: String,
        pub name: String,
        pub size: i64,
        pub date_taken: i64,
        pub mime: Option<String>,
        pub width: Option<i32>,
        pub height: Option<i32>,
        pub duration_ms: Option<i64>,
    }

    /// One page of a bucket, newest first; `next` is the cursor of the next page.
    #[derive(Debug, Clone, Deserialize)]
    pub struct AssetPage {
        pub items: Vec<Asset>,
        pub next: Option<String>,
    }

    /// Why the platform refused to answer.
    #[derive(Debug)]
    pub enum Error {
        /// `init` has not run, so there is no bridge to ask.
        NotRegistered,
        /// The user did not grant, or revoked, access to the library.
        PermissionDenied,
        /// The query itself failed; the message is the platform's own.
        Platform(String),
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NotRegistered => f.write_str("MediaStore 插件未注册"),
                Self::PermissionDenied => f.write_str("未授予相册权限"),
                Self::Platform(cause) => f.write_str(cause),
            }
        }
    }

    impl std::error::Error for Error {}

    #[derive(Serialize)]
    struct UriArgs<'a> {
        uri: &'a str,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct AssetPageArgs<'a> {
        uri: &'a str,
        cursor: Option<&'a str>,
        limit: u32,
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

    #[derive(Deserialize)]
    struct Buckets {
        entries: Vec<Entry>,
    }

    async fn call<T>(command: &str, payload: impl Serialize) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let handle = HANDLE.get().ok_or(Error::NotRegistered)?;
        handle
            .run_mobile_plugin_async(command, payload)
            .await
            .map_err(rejected)
    }

    /// What the platform allows right now.
    pub async fn permission_state() -> Result<PermissionState, Error> {
        call("permissionState", ()).await
    }

    /// Ask for access, opening the system dialog when it can still appear.
    pub async fn request_permission() -> Result<PermissionState, Error> {
        call("requestPermission", ()).await
    }

    /// The flat bucket list; asking for it is what opens the permission dialog.
    pub async fn browse() -> Result<Vec<Entry>, Error> {
        let buckets: Buckets = call("browse", ()).await?;
        Ok(buckets.entries)
    }

    /// One page of a bucket, newest first. `cursor` is the `next` of the
    /// previous page, `None` for the first.
    pub async fn assets(
        album_uri: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<AssetPage, Error> {
        call(
            "assets",
            AssetPageArgs {
                uri: album_uri,
                cursor,
                limit,
            },
        )
        .await
    }

    /// The row behind `id`, or `None` when MediaStore no longer has it - a
    /// gallery deletion between a page and the tap on one of its rows.
    pub async fn asset(id: &str) -> Result<Option<Asset>, Error> {
        call("asset", AssetArgs { id }).await
    }

    /// How many rows the bucket holds.
    pub async fn count(album_uri: &str) -> Result<u32, Error> {
        call("count", UriArgs { uri: album_uri }).await
    }

    /// The row's thumbnail as JPEG bytes, square, `edge` pixels a side.
    pub async fn thumb(id: &str, edge: u32) -> Result<Vec<u8>, Error> {
        let encoded: String = call("thumb", ThumbArgs { id, edge }).await?;
        STANDARD
            .decode(encoded)
            .map_err(|error| Error::Platform(format!("无法解码缩略图: {error}")))
    }

    /// A rejection carries the message the user should see, and a permission
    /// one is a different situation from a query that blew up, so its code
    /// survives.
    fn rejected(error: PluginInvokeError) -> Error {
        match error {
            PluginInvokeError::InvokeRejected(response) => {
                let message = response
                    .message
                    .unwrap_or_else(|| "无法访问系统相册".to_string());
                match response.code.as_deref() {
                    Some("permission" | "denied") => Error::PermissionDenied,
                    _ => Error::Platform(message),
                }
            }
            other => Error::Platform(format!("无法访问系统相册: {other}")),
        }
    }
}

#[cfg(target_os = "android")]
pub use android::{
    Asset, AssetPage, Entry, Error, Grant, PermissionState, asset, assets, browse, count, init,
    permission_state, request_permission, thumb,
};
