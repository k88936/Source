//! Where a Cloudreve token lives between launches, one per account.
//!
//! The use cases only know that a token can be read, written and cleared for a
//! server; the platform decides whether that is the OS keyring or a file.

pub trait CredentialStore: Send + Sync {
    fn read(&self, key: &str) -> Option<String>;
    fn write(&self, key: &str, value: &str);
    fn clear(&self, key: &str);
}
