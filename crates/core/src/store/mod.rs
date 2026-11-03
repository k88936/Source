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

mod album;
mod asset;
mod dto;
mod folder;
mod local;
mod query;
mod queue;
mod rows;
pub(crate) mod schema;
mod server;
mod timeline;
pub(crate) mod values;

pub(crate) use super::sqlite::{Conn, MEDIA_TYPES, SqliteCatalog, TxError, db_err};
pub use album::*;
pub use asset::*;
pub use dto::*;
pub use folder::*;
pub use queue::*;
pub use server::*;
pub use timeline::*;
