//! Diesel-backed SQLite implementation of [`CatalogPort`] and
//! [`UploadSessionPort`].
//!
//! The schema lives in `migrations/` and is mirrored by `schema.rs`; every query
//! is written with the diesel DSL so column types, joins and upserts are checked
//! at compile time. Migrations are applied once at startup, which replaces the
//! previous hand-written `PRAGMA user_version` handling entirely.

use std::path::Path;

use async_trait::async_trait;
use diesel::expression_methods::EscapeExpressionMethods;
use diesel::prelude::*;
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel::upsert::excluded;
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::{
    AsyncDieselConnectionManager, ManagerConfig, RecyclingMethod,
};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, RunQueryDsl, SimpleAsyncConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use source_core::dto::{
    AlbumId, AlbumSummary, AppError, AssetId, AssetSummary, FolderId, FolderRef, LocalAsset,
    MediaType, TakenAtSource, TimelineCursor, TimelinePage, UploadState, UploadTask, UploadTaskId,
};
use source_core::ports::{
    AssetRecord, CatalogPort, FolderRecord, FolderSyncState, SearchQuery, StoredJournal,
    StoredRecord, StoredServer, StoredSession, UploadSessionPort,
};

pub mod schema;

mod albums;
mod assets;
mod folders;
mod local;
mod port;
mod query;
mod rows;
mod server;
mod sessions;
mod uploads;
mod values;

/// Connections handed out by the pool.
type Conn = Object<SyncConnectionWrapper<SqliteConnection>>;

/// Media types that appear in the timeline and in albums.
const MEDIA_TYPES: [&str; 2] = ["image", "video"];

/// Pragmas applied to every pooled connection.
const PRAGMAS: &str =
    "PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;";

/// SQLite pool size. SQLite serialises writers, so a handful is plenty.
const POOL_SIZE: usize = 4;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct SqliteCatalog {
    pool: Pool<SyncConnectionWrapper<SqliteConnection>>,
}

impl SqliteCatalog {
    /// Open (or create) the catalog at `path`, applying pending migrations.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let url = path.as_ref().to_string_lossy().into_owned();
        let migrate_url = url.clone();
        tokio::task::spawn_blocking(move || migrate(&migrate_url))
            .await
            .map_err(|e| AppError::internal(format!("catalog migrate: {e}")))??;
        Self::from_url(url)
    }

    fn from_url(url: String) -> Result<Self, AppError> {
        let mut config = ManagerConfig::<SyncConnectionWrapper<SqliteConnection>>::default();
        config.recycling_method = RecyclingMethod::Fast;
        config.custom_setup = Box::new(|url: &str| {
            let url = url.to_owned();
            Box::pin(async move {
                let mut conn = SyncConnectionWrapper::<SqliteConnection>::establish(&url).await?;
                SimpleAsyncConnection::batch_execute(&mut conn, PRAGMAS)
                    .await
                    .map_err(|e| diesel::result::ConnectionError::BadConnection(e.to_string()))?;
                Ok(conn)
            })
        });
        let manager = AsyncDieselConnectionManager::new_with_config(url, config);
        let pool = Pool::builder(manager)
            .max_size(POOL_SIZE)
            .build()
            .map_err(|e| AppError::internal(format!("catalog pool: {e}")))?;
        Ok(Self { pool })
    }

    async fn conn(&self) -> Result<Conn, AppError> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::internal(format!("catalog: {e}")))
    }
}

fn migrate(url: &str) -> Result<(), AppError> {
    let mut conn = SqliteConnection::establish(url).map_err(conn_err)?;
    diesel::connection::SimpleConnection::batch_execute(&mut conn, PRAGMAS).map_err(db_err)?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| AppError::internal(format!("catalog: {e}")))?;
    Ok(())
}

fn db_err(err: diesel::result::Error) -> AppError {
    AppError::internal(format!("catalog: {err}"))
}

fn conn_err(err: diesel::result::ConnectionError) -> AppError {
    AppError::internal(format!("catalog: {err}"))
}

/// Transaction bodies must fail with an error diesel can convert into a
/// rollback, which `AppError` cannot express on its own.
struct TxError(AppError);

impl From<diesel::result::Error> for TxError {
    fn from(err: diesel::result::Error) -> Self {
        Self(db_err(err))
    }
}

impl From<AppError> for TxError {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}
