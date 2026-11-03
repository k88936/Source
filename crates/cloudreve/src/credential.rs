//! Credential storage shared by the keyring and filesystem stores.

use super::*;

pub trait CredentialStore: Send + Sync {
    fn read(&self) -> Option<String>;
    fn write(&self, value: &str);
    fn clear(&self);
}

/// Filesystem fallback/migration store. The Tauri app installs a keyring-backed
/// implementation (`tauri-plugin-keyring`) in front of this.
pub struct FileCredentialStore {
    path: PathBuf,
}

impl FileCredentialStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl CredentialStore for FileCredentialStore {
    fn read(&self) -> Option<String> {
        fs::read_to_string(&self.path).ok()
    }

    fn write(&self, value: &str) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let tmp = self.path.with_extension("tmp");
        if fs::write(&tmp, value).is_ok() {
            let _ = fs::rename(&tmp, &self.path);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600));
            }
        }
    }

    fn clear(&self) {
        let _ = fs::remove_file(&self.path);
    }
}
