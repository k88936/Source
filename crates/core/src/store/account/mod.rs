//! The account a library belongs to: the contract and the diesel statements
//! behind it.
//!
//! The catalog indexes one account at a time, so every read goes through
//! [`AccountStore::current_account`] instead of keeping a copy of the server or
//! of the library root anywhere else.

use async_trait::async_trait;

use self::rows::AccountRow;
use self::schema::account;
use super::settings::schema::app_state;
use super::*;
use crate::time::now_ms;

mod dto;
pub(crate) mod rows;
pub(crate) mod schema;

pub use dto::*;

/// The setting pointing at the account the UI works on.
pub(crate) const CURRENT_ACCOUNT: &str = "current_account";

#[async_trait]
pub trait AccountStore: Send + Sync {
    /// The account `current_account` points at, so `None` means "no library
    /// chosen yet".
    async fn current_account(&self) -> Result<Option<Account>, AppError>;
    /// The account remembered for `base_url`, if that server was used before.
    async fn account(&self, base_url: &str) -> Result<Option<Account>, AppError>;
    /// Insert or update the row and make it the current account.
    async fn save_account(&self, value: &Account) -> Result<(), AppError>;
    /// Point the library of `base_url` at `root`.
    async fn set_library_root(&self, base_url: &str, root: &FolderRef) -> Result<(), AppError>;
}

pub(crate) async fn current_account(cat: &SqliteCatalog) -> Result<Option<Account>, AppError> {
    let mut conn = cat.conn().await?;
    let base_url = app_state::table
        .filter(app_state::key.eq(CURRENT_ACCOUNT))
        .select(app_state::value)
        .first::<String>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    let Some(base_url) = base_url else {
        return Ok(None);
    };
    let row = account_row(&mut conn, &base_url).await?;
    row.map(AccountRow::into_account).transpose()
}

pub(crate) async fn account(
    cat: &SqliteCatalog,
    base_url: &str,
) -> Result<Option<Account>, AppError> {
    let mut conn = cat.conn().await?;
    let row = account_row(&mut conn, base_url).await?;
    row.map(AccountRow::into_account).transpose()
}

async fn account_row(conn: &mut Conn, base_url: &str) -> Result<Option<AccountRow>, AppError> {
    account::table
        .filter(account::base_url.eq(base_url))
        .select((
            account::base_url,
            account::user,
            account::site_name,
            account::library_root,
        ))
        .first::<AccountRow>(conn)
        .await
        .optional()
        .map_err(db_err)
}

pub(crate) async fn save_account(cat: &SqliteCatalog, value: &Account) -> Result<(), AppError> {
    let user = rows::encode(&value.user)?;
    let library_root = rows::encode(&value.library_root)?;
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::insert_into(account::table)
            .values((
                account::base_url.eq(&value.base_url),
                account::user.eq(&user),
                account::site_name.eq(&value.site_name),
                account::library_root.eq(&library_root),
                account::created_at.eq(now_ms()),
            ))
            .on_conflict(account::base_url)
            .do_update()
            .set((
                account::user.eq(excluded(account::user)),
                account::site_name.eq(excluded(account::site_name)),
                account::library_root.eq(excluded(account::library_root)),
            ))
            .execute(&mut *conn)
            .await?;
        diesel::insert_into(app_state::table)
            .values((
                app_state::key.eq(CURRENT_ACCOUNT),
                app_state::value.eq(&value.base_url),
            ))
            .on_conflict(app_state::key)
            .do_update()
            .set(app_state::value.eq(excluded(app_state::value)))
            .execute(&mut *conn)
            .await?;
        Ok(())
    })
    .await
    .map_err(|error| error.0)?;
    Ok(())
}

pub(crate) async fn set_library_root(
    cat: &SqliteCatalog,
    base_url: &str,
    root: &FolderRef,
) -> Result<(), AppError> {
    let root = serde_json::to_string(root)
        .map_err(|error| AppError::internal(format!("无法序列化图库根: {error}")))?;
    let mut conn = cat.conn().await?;
    diesel::update(account::table.filter(account::base_url.eq(base_url)))
        .set(account::library_root.eq(root))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

#[async_trait]
impl AccountStore for SqliteCatalog {
    async fn current_account(&self) -> Result<Option<Account>, AppError> {
        current_account(self).await
    }

    async fn account(&self, base_url: &str) -> Result<Option<Account>, AppError> {
        account(self, base_url).await
    }

    async fn save_account(&self, value: &Account) -> Result<(), AppError> {
        save_account(self, value).await
    }

    async fn set_library_root(&self, base_url: &str, root: &FolderRef) -> Result<(), AppError> {
        set_library_root(self, base_url, root).await
    }
}
