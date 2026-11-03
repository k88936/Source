//! Account tokens: the platform keychain, plus a plaintext file for dev builds.
//!
//! Both stores are keyed by the account's server, so two accounts can be signed
//! in without overwriting each other's token.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use source_core::CredentialStore;
use tauri::AppHandle;
use tauri_plugin_keyring::{CredentialType, CredentialValue, KeyringExt};

/// Tokens in the OS keychain (Android Keystore, Apple Keychain, Windows
/// Credential Manager, Secret Service), one entry per server.
pub struct KeyringCredentialStore {
    app: AppHandle,
    available: AtomicBool,
}

impl KeyringCredentialStore {
    const SERVICE: &'static str = "top.k88936.source";

    pub fn new(app: AppHandle) -> Self {
        let available = app
            .keyring()
            .initialize_service(Self::SERVICE.to_string())
            .is_ok();
        if !available {
            eprintln!("keyring unavailable; session will not survive a restart");
        }
        Self {
            app,
            available: AtomicBool::new(available),
        }
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn read(&self, key: &str) -> Option<String> {
        if !self.available.load(Ordering::Relaxed) {
            return None;
        }
        match self.app.keyring().get(key, CredentialType::Password) {
            Ok(CredentialValue::Password(value)) => Some(value),
            _ => None,
        }
    }

    fn write(&self, key: &str, value: &str) {
        if !self.available.load(Ordering::Relaxed) {
            eprintln!("platform keyring unavailable; session will not be persisted");
            return;
        }
        if let Err(error) = self.app.keyring().set(
            key,
            CredentialType::Password,
            CredentialValue::Password(value.to_string()),
        ) {
            eprintln!("failed to write session to platform keyring: {error}");
        }
    }

    fn clear(&self, key: &str) {
        if self.available.load(Ordering::Relaxed) {
            let _ = self.app.keyring().delete(key, CredentialType::Password);
        }
    }
}

/// Filesystem store, used by dev builds.
pub struct FileCredentialStore {
    dir: PathBuf,
}

impl FileCredentialStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The server URL is not a legal file name on every platform, so the key is
    /// hex-encoded rather than sanitized.
    fn path(&self, key: &str) -> PathBuf {
        let mut name = String::with_capacity(key.len() * 2);
        for byte in key.as_bytes() {
            name.push_str(&format!("{byte:02x}"));
        }
        self.dir.join(name)
    }
}

impl CredentialStore for FileCredentialStore {
    fn read(&self, key: &str) -> Option<String> {
        std::fs::read_to_string(self.path(key)).ok()
    }

    fn write(&self, key: &str, value: &str) {
        if std::fs::create_dir_all(&self.dir).is_err() {
            return;
        }
        let path = self.path(key);
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, value).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }

    fn clear(&self, key: &str) {
        let _ = std::fs::remove_file(self.path(key));
    }
}
