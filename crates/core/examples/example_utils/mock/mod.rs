//! In-memory doubles for the ports that otherwise reach the platform.
//!
//! They keep their state for one process: enough to drive a use case where a
//! keyring or a file would only be in the way.

pub mod credential_store;
