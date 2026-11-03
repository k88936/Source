//! Session tokens in the OS keychain.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use source_cloudreve::{CredentialStore, FileCredentialStore};
use tauri::AppHandle;
use tauri_plugin_keyring::{CredentialType, CredentialValue, KeyringExt};

/// Session tokens in the OS keychain (Android Keystore, Apple Keychain,
/// Windows Credential Manager, Secret Service). A legacy 0600 file is read
/// once and deleted after migration; new tokens are never written in plaintext.
pub struct KeyringCredentialStore {
    app: AppHandle,
    fallback: FileCredentialStore,
    available: AtomicBool,
}

impl KeyringCredentialStore {
    const SERVICE: &'static str = "top.k88936.source";
    const USER: &'static str = "cloudreve-session";

    pub fn new(app: AppHandle, legacy_path: PathBuf) -> Self {
        let available = app
            .keyring()
            .initialize_service(Self::SERVICE.to_string())
            .is_ok();
        if !available {
            eprintln!("keyring unavailable; session will not survive a restart");
        }
        Self {
            app,
            fallback: FileCredentialStore::new(legacy_path),
            available: AtomicBool::new(available),
        }
    }

    fn keyring_read(&self) -> Option<String> {
        if !self.available.load(Ordering::Relaxed) {
            return None;
        }
        match self.app.keyring().get(Self::USER, CredentialType::Password) {
            Ok(CredentialValue::Password(value)) => Some(value),
            _ => None,
        }
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn read(&self) -> Option<String> {
        if let Some(value) = self.keyring_read() {
            return Some(value);
        }
        if !cfg!(desktop) {
            return None;
        }
        let legacy = self.fallback.read();
        if let Some(value) = &legacy
            && self.available.load(Ordering::Relaxed)
            && self
                .app
                .keyring()
                .set(
                    Self::USER,
                    CredentialType::Password,
                    CredentialValue::Password(value.clone()),
                )
                .is_ok()
        {
            self.fallback.clear();
        }
        legacy
    }

    fn write(&self, value: &str) {
        if self.available.load(Ordering::Relaxed) {
            if let Err(error) = self.app.keyring().set(
                Self::USER,
                CredentialType::Password,
                CredentialValue::Password(value.to_string()),
            ) {
                // Never downgrade to a plaintext file; the user simply has to
                // log in again next launch.
                eprintln!("failed to write session to platform keyring: {error}");
            }
            self.fallback.clear();
            return;
        }
        eprintln!("platform keyring unavailable; session will not be persisted");
    }

    fn clear(&self) {
        if self.available.load(Ordering::Relaxed) {
            let _ = self
                .app
                .keyring()
                .delete(Self::USER, CredentialType::Password);
        }
        self.fallback.clear();
    }
}
