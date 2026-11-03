//! Development-only wiring: an isolated home, the `tmp/` hand-off files written
//! by `scripts/dev_cloudreve.py` and the Vite dev server, plus the simulated CDP
//! endpoint. Everything here is compiled out of release builds.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::state::App;
use clap::Parser;
use serde::Deserialize;
use source_cloudreve::{CredentialStore, FileCredentialStore};
use source_core::LoginOutcome;
use tauri::Manager;

/// Where the dev scripts hand their results over. `SOURCE_DEV_TMP` lets a shell
/// point the app at another checkout; the default is this repository's `tmp/`
/// (the app's cwd is `src-tauri/`, so a relative path would not do).
pub(crate) fn tmp_dir() -> PathBuf {
    match std::env::var_os("SOURCE_DEV_TMP") {
        Some(path) => PathBuf::from(path),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("tmp"),
    }
}

/// Keep every byte the app writes inside `<repo>/tmp/home`.
///
/// `HOME` and `XDG_RUNTIME_DIR` are deliberately untouched: they are the handles
/// to the desktop session (Wayland socket, D-Bus), and redirecting them breaks it.
pub(crate) fn isolate_home() {
    let root = tmp_dir().join("home");
    for (variable, path) in [
        ("XDG_DATA_HOME", root.join(".local/share")),
        ("XDG_CONFIG_HOME", root.join(".config")),
        ("XDG_CACHE_HOME", root.join(".cache")),
        ("XDG_STATE_HOME", root.join(".local/state")),
    ] {
        if let Err(error) = std::fs::create_dir_all(&path) {
            eprintln!("dev: cannot create {}: {error}", path.display());
            continue;
        }
        // SAFETY: called as the first statement of `run`, before any thread or
        // library reads the environment.
        unsafe { std::env::set_var(variable, &path) };
    }
    println!("dev: XDG home isolated to {}", root.display());
}

/// Session tokens of a dev build live in `tmp/home`, next to the catalog, instead
/// of the shared OS keychain — logging in here never disturbs a real session.
pub(crate) fn credential_store(app_data: &Path) -> Arc<dyn CredentialStore> {
    Arc::new(FileCredentialStore::new(app_data.join("credentials.json")))
}

/// `--debug-cdp-port <port>`: absent means no CDP endpoint at all.
pub(crate) fn cdp_port() -> Option<u16> {
    let port = DevArgs::parse().cdp_port?;
    if let Err(error) = TcpListener::bind(("127.0.0.1", port)) {
        eprintln!("dev: cannot listen on 127.0.0.1:{port}: {error}");
        std::process::exit(2);
    }
    println!("dev: simulated CDP will listen on http://127.0.0.1:{port}");
    Some(port)
}

/// The plugin's Unix socket is `/tmp/tauri-playwright.sock` for every instance and
/// it unlinks whatever sits there before binding, so a second dev build silently
/// takes the socket away from the first one. Give each run its own name and
/// publish it in `playwright.json`, the way the other dev hand-off files work.
pub(crate) fn playwright_config(port: u16) -> tauri_plugin_playwright::PluginConfig {
    let socket = socket_path();
    let info = serde_json::json!({
        "socket": socket.to_string_lossy(),
        "cdp": format!("http://127.0.0.1:{port}"),
        "pid": std::process::id(),
    });
    let path = tmp_dir().join("playwright.json");
    match std::fs::write(&path, info.to_string()) {
        Ok(()) => println!("dev: playwright socket at {}", socket.display()),
        Err(error) => eprintln!("dev: cannot write {}: {error}", path.display()),
    }
    tauri_plugin_playwright::PluginConfig::new()
        .cdp_port(port)
        .socket_path(socket.to_string_lossy())
}

/// A name no other run of this machine will pick: `RandomState` seeds itself from
/// the operating system, so the hash of a constant differs per process.
fn socket_path() -> PathBuf {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u8(0);
    let name = format!("tauri-playwright-{:016x}.sock", hasher.finish());

    // Canonical, because this path ends up in the plugin's own log line and in
    // `playwright.json`; `tmp_dir()` still carries the `src-tauri/..` hop.
    let dir = tmp_dir();
    let _ = std::fs::create_dir_all(&dir);
    let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
    #[cfg(unix)]
    remove_stale_sockets(&dir);
    let path = dir.join(&name);
    // A Unix socket path is limited to about 108 bytes, which a deep checkout
    // under an exotic `SOURCE_DEV_TMP` can exceed.
    if path.to_string_lossy().len() > 100 {
        eprintln!(
            "dev: {} is too long for a socket; using /tmp",
            path.display()
        );
        return std::env::temp_dir().join(name);
    }
    path
}

/// A socket file outlives the process that bound it, so random names would pile
/// up one dead file per run. Whatever nothing accepts a connection on is stale;
/// a live instance's socket stays.
#[cfg(unix)]
fn remove_stale_sockets(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("tauri-playwright-") || !name.ends_with(".sock") {
            continue;
        }
        let path = entry.path();
        if std::os::unix::net::UnixStream::connect(&path).is_err() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// Dev-only command line; release builds parse nothing and accept no flags.
#[derive(Parser)]
#[command(name = "source", about = "Source (development build)")]
struct DevArgs {
    /// Listen for the simulated Chrome DevTools Protocol on this port.
    #[arg(long = "debug-cdp-port", value_name = "PORT")]
    cdp_port: Option<u16>,
}

#[derive(Deserialize)]
struct ViteInfo {
    url: String,
}

#[derive(Deserialize)]
struct CloudreveConfig {
    endpoint: String,
    email: String,
    password: String,
    uri: String,
}

fn read_json<T: for<'de> Deserialize<'de>>(name: &str) -> Option<T> {
    let path = tmp_dir().join(name);
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) => {
            println!("dev: {} unreadable ({error}); skipping", path.display());
            return None;
        }
    };
    match serde_json::from_str(&raw) {
        Ok(value) => Some(value),
        Err(error) => {
            eprintln!("dev: {} is not valid JSON: {error}", path.display());
            None
        }
    }
}

/// Point the window at whatever port Vite picked, so the dev server never has to
/// compete for `build.devUrl`. The moved origin is a remote one for Tauri, which
/// is why `capabilities/dev.json`/`dev-remote.json` grant the app commands.
pub(crate) fn open_dev_frontend(app: &tauri::AppHandle) {
    let Some(vite) = read_json::<ViteInfo>("vite.json") else {
        return;
    };
    let url = match tauri::Url::parse(vite.url.trim_end_matches('/')) {
        Ok(url) => url,
        Err(error) => {
            eprintln!("dev: {} is not a URL: {error}", vite.url);
            return;
        }
    };
    let Some(window) = app.get_webview_window("main") else {
        eprintln!("dev: no main window to point at {url}");
        return;
    };
    if window.url().ok().as_ref() == Some(&url) {
        println!("dev: frontend at {url}");
        return;
    }
    match window.navigate(url.clone()) {
        Ok(()) => println!("dev: window follows the frontend to {url}"),
        Err(error) => eprintln!("dev: cannot load {url}: {error}"),
    }
}

/// Log in with the credentials `scripts/dev_cloudreve.py` wrote, so a freshly
/// provisioned container needs no manual setup. Failures never block startup.
pub(crate) fn autoconnect(core: &App) {
    let Some(config) = read_json::<CloudreveConfig>("cloudreve.json") else {
        return;
    };
    if core
        .session_state()
        .is_some_and(|session| session.server == config.endpoint)
    {
        println!("dev: already connected to {}", config.endpoint);
        return;
    }
    tauri::async_runtime::block_on(async {
        if let Err(error) = core.set_server(&config.endpoint).await {
            eprintln!("dev: cannot reach {}: {error}", config.endpoint);
            return;
        }
        match core.login(&config.email, &config.password).await {
            Ok(LoginOutcome::Authenticated { .. }) => {}
            Ok(LoginOutcome::TwoFactorRequired { .. }) => {
                eprintln!(
                    "dev: {} needs a two-factor code; log in manually",
                    config.email
                );
                return;
            }
            Err(error) => {
                eprintln!("dev: login as {} failed: {error}", config.email);
                return;
            }
        }
        let has_root = core
            .session_state()
            .and_then(|session| session.library_root)
            .is_some();
        if !has_root && let Err(error) = core.set_library_root(&config.uri).await {
            eprintln!(
                "dev: cannot use {} as the library root: {error}",
                config.uri
            );
        }
        println!("dev: connected to {} as {}", config.endpoint, config.email);
    });
}
