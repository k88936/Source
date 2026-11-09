//! Kotlin bridge for the keep-alive foreground service.
//!
//! Android freezes, then kills, a background process, so work that outlives the
//! window needs a foreground service, and a partial wake lock keeps the CPU up
//! under it while the screen is off. Nothing reaches this side but one fact -
//! whether to hold - so the caller never learns what the platform charges for it.

#[cfg(target_os = "android")]
mod android {
    use std::sync::OnceLock;

    use serde::Serialize;
    use tauri::Wry;
    use tauri::plugin::PluginHandle;

    /// Where the Kotlin class lives: the package it declares, then the class name.
    const PACKAGE: &str = "top.k88936.source.mobile.keepalive";
    const CLASS: &str = "KeepAlivePlugin";

    /// The plugin's name on both sides of the bridge; the one command is reached
    /// through it.
    const PLUGIN: &str = "keep-alive";

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

    #[derive(Serialize)]
    struct SyncArgs {
        hold: bool,
    }

    /// Tell the platform whether to hold the process.
    ///
    /// The caller only reports a state, so the call is idempotent: a lost one is
    /// corrected by the next, and the platform is never left holding a process for
    /// work that was paused.
    pub async fn sync(hold: bool) {
        let handle = HANDLE.get().expect("KeepAlive 插件未注册");
        if let Err(error) = handle
            .run_mobile_plugin_async::<()>("sync", SyncArgs { hold })
            .await
        {
            eprintln!("keep-alive sync failed: {error}");
        }
    }
}

#[cfg(target_os = "android")]
pub use android::{init, sync};
