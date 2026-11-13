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
mod observe;
mod query;
mod settings;
mod timeline;
mod upload_queue;
mod upload_session;

pub(crate) use super::sqlite::{Conn, MEDIA_TYPES, SqliteCatalog, TxError, db_err};
pub(crate) use asset::schema::{
    ASSET_FOLDER_ID, ASSET_ID, ASSET_MEDIA_TYPE, ASSET_TAKEN_AT, ASSETS,
};
pub(crate) use folder::schema::{FOLDER_ID, FOLDERS, SYNC_FOLDER_ID, SYNC_STATE};
pub use observe::observe_commits;
pub(crate) use observe::{install, resolve};

/// Every table column a view declared, as one read set.
///
/// The observer may only pull what a view reads out of a row, so this is the
/// union of what the view modules asked for. Widening it is cheap; a table
/// missing here is a view that never hears about a commit.
pub(crate) static READ_SET: [source_subscription::Table; 3] = [
    folder::schema::FOLDERS,
    folder::schema::SYNC_STATE,
    asset::schema::ASSETS,
];

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
pub use asset::*;
pub use error::CatalogError;
pub use folder::*;
pub(crate) use id::id_type;
pub use local::*;
pub use settings::*;
pub use timeline::*;
pub use upload_queue::*;
pub use upload_session::*;
