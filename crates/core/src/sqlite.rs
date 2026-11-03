//! The pooled SQLite connection every store module talks to.
//!
//! The schema lives in `migrations/` and is mirrored by [`crate::store::schema`];
//! every query is written with the diesel DSL so column types, joins and upserts
//! are checked at compile time. Pending migrations are applied once, at startup.

use std::path::Path;

use diesel::Connection;
use diesel::sqlite::SqliteConnection;
use diesel_async::AsyncConnection;
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::{
    AsyncDieselConnectionManager, ManagerConfig, RecyclingMethod,
};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::error::AppError;

/// Connections handed out by the pool.
pub(crate) type Conn = Object<SyncConnectionWrapper<SqliteConnection>>;

/// Media types that appear in the timeline and in albums.
pub(crate) const MEDIA_TYPES: [&str; 2] = ["image", "video"];

/// Pragmas applied to every pooled connection.
const PRAGMAS: &str =
    "PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;";

/// SQLite pool size. SQLite serialises writers, so a handful is plenty.
const POOL_SIZE: usize = 4;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
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
                diesel_async::SimpleAsyncConnection::batch_execute(&mut conn, PRAGMAS)
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

    pub(crate) async fn conn(&self) -> Result<Conn, AppError> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::internal(format!("catalog: {e}")))
    }
}

fn migrate(url: &str) -> Result<(), AppError> {
    use diesel::connection::SimpleConnection;

    let mut conn = SqliteConnection::establish(url).map_err(conn_err)?;
    conn.batch_execute(PRAGMAS).map_err(db_err)?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| AppError::internal(format!("catalog: {e}")))?;
    Ok(())
}

pub(crate) fn db_err(err: diesel::result::Error) -> AppError {
    AppError::internal(format!("catalog: {err}"))
}

fn conn_err(err: diesel::result::ConnectionError) -> AppError {
    AppError::internal(format!("catalog: {err}"))
}

/// Transaction bodies must fail with an error diesel can convert into a
/// rollback, which `AppError` cannot express on its own.
pub(crate) struct TxError(pub(crate) AppError);

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
