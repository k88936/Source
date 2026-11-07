//! The SQLite adapter: the store contracts and the diesel statements behind
//! them, one module per family of rows.
//!
//! Each module owns one family - its payload types, its contract and the
//! statements - so a use-case module names exactly the parts of the index it is
//! allowed to touch as bounds on its `impl` block. The shared glue (pool,
//! pragmas, migrations, error mapping) lives in [`super::sqlite`].

use diesel::expression_methods::EscapeExpressionMethods;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::upsert::excluded;
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::media::LocalAsset;

mod account;
mod album;
mod asset;
mod error;
mod folder;
mod id;
mod local;
mod query;
mod settings;
mod timeline;
mod upload_queue;
mod upload_session;

pub(crate) use super::sqlite::{Conn, MEDIA_TYPES, SqliteCatalog, TxError, db_err};

/// The address Cloudreve gives the drive root.
const DRIVE_ROOT_URI: &str = "cloudreve://my";

/// The path the UI shows: relative to the drive root and decoded, so
/// `cloudreve://my/%E9%AA%8C%E8%AF%81` reads `/验证`. The `folders.path` column
/// keeps the server's form, and rows are converted when they leave the store.
pub(crate) fn display_path(path: &str) -> String {
    let relative = path.strip_prefix(DRIVE_ROOT_URI).unwrap_or(path);
    String::from_utf8_lossy(&urlencoding::decode_binary(relative.as_bytes())).into_owned()
}

pub use account::*;
pub use album::*;
pub use error::CatalogError;
pub use asset::*;
pub use folder::*;
pub(crate) use id::id_type;
pub use local::*;
pub use settings::*;
pub use timeline::*;
pub use upload_queue::*;
pub use upload_session::*;
