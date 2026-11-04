//! A token store that lives as long as the process.

use std::collections::HashMap;
use std::sync::Mutex;

use source_core::CredentialStore;

/// A token held in memory: for callers that have no keyring and nothing to
/// keep between runs.
#[derive(Default)]
pub struct MemoryCredentials {
    tokens: Mutex<HashMap<String, String>>,
}

impl CredentialStore for MemoryCredentials {
    fn read(&self, key: &str) -> Option<String> {
        self.tokens.lock().unwrap().get(key).cloned()
    }

    fn write(&self, key: &str, value: &str) {
        self.tokens
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }

    fn clear(&self, key: &str) {
        self.tokens.lock().unwrap().remove(key);
    }
}
