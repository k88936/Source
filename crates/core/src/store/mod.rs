//! The SQLite adapter: the store contracts and the diesel statements behind
//! them, one module per family of rows.
//!
//! Each module owns one family — its payload types, its contract and the
//! statements — so a use-case module names exactly the parts of the index it is
//! allowed to touch as bounds on its `impl` block. The shared glue (pool,
//! pragmas, migrations, error mapping) lives in [`super::sqlite`].

use diesel::expression_methods::EscapeExpressionMethods;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::upsert::excluded;
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::error::AppError;
use crate::media::LocalAsset;

mod account;
mod album;
mod asset;
mod folder;
mod id;
mod local;
mod query;
mod queue;
mod settings;
mod timeline;
mod upload_session;

pub(crate) use super::sqlite::{Conn, MEDIA_TYPES, SqliteCatalog, TxError, db_err};
pub use account::*;
pub use album::*;
pub use asset::*;
pub use folder::*;
pub(crate) use id::id_type;
pub use local::*;
pub use queue::*;
pub use settings::*;
pub use timeline::*;
pub use upload_session::*;
