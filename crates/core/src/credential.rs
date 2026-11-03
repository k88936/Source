//! Where the Cloudreve session token lives between launches.
//!
//! The use cases only know that a token can be read, written and cleared; the
//! platform decides whether that is the OS keyring or a file.

pub trait CredentialStore: Send + Sync {
    fn read(&self) -> Option<String>;
    fn write(&self, value: &str);
    fn clear(&self);
}
