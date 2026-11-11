//! The account a library belongs to: the contract and the diesel statements
//! behind it.
//!
//! The catalog indexes one account at a time, so every read goes through
//! [`AccountStore::get_current_account`] instead of keeping a copy of the server
//! anywhere else.

use async_trait::async_trait;

use self::rows::AccountRow;
use self::schema::account;
use super::asset::schema::assets;
use super::folder::schema::{folders, sync_state};
use super::local::schema::{local_album_mapping, local_albums};
use super::settings::schema::app_state;
use super::upload_queue::schema::upload_queue;
use super::upload_session::schema::upload_sessions;
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
    async fn get_current_account(&self) -> Result<Option<Account>, CatalogError>;
    /// The account remembered for `base_url`, if that server was used before.
    async fn get_account(&self, base_url: &str) -> Result<Option<Account>, CatalogError>;
    /// Insert or update the row and make it the current account.
    async fn save_account(&self, value: &Account) -> Result<(), CatalogError>;
    /// Drop everything that belongs to the previous server.
    ///
    /// The catalog is single-account: albums, their assets, the local album
    /// registry and its mapping all point at folders of the server the current
    /// account lives on, so they cannot survive a different one.
    async fn reset_catalog(&self) -> Result<(), CatalogError>;
}

pub(crate) async fn get_current_account(
    cat: &SqliteCatalog,
) -> Result<Option<Account>, CatalogError> {
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

pub(crate) async fn get_account(
    cat: &SqliteCatalog,
    base_url: &str,
) -> Result<Option<Account>, CatalogError> {
    let mut conn = cat.conn().await?;
    let row = account_row(&mut conn, base_url).await?;
    row.map(AccountRow::into_account).transpose()
}

async fn account_row(conn: &mut Conn, base_url: &str) -> Result<Option<AccountRow>, CatalogError> {
    account::table
        .filter(account::base_url.eq(base_url))
        .select((account::base_url, account::user, account::site_name))
        .first::<AccountRow>(conn)
        .await
        .optional()
        .map_err(db_err)
}

pub(crate) async fn save_account(cat: &SqliteCatalog, value: &Account) -> Result<(), CatalogError> {
    let user = rows::encode(&value.user)?;
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::insert_into(account::table)
            .values((
                account::base_url.eq(&value.base_url),
                account::user.eq(&user),
                account::site_name.eq(&value.site_name),
                account::created_at.eq(now_ms()),
            ))
            .on_conflict(account::base_url)
            .do_update()
            .set((
                account::user.eq(excluded(account::user)),
                account::site_name.eq(excluded(account::site_name)),
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

pub(crate) async fn reset_catalog(cat: &SqliteCatalog) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    conn.transaction::<(), TxError, _>(async |conn| {
        diesel::delete(upload_sessions::table)
            .execute(&mut *conn)
            .await?;
        diesel::delete(upload_queue::table)
            .execute(&mut *conn)
            .await?;
        diesel::delete(local_album_mapping::table)
            .execute(&mut *conn)
            .await?;
        diesel::delete(local_albums::table)
            .execute(&mut *conn)
            .await?;
        diesel::delete(sync_state::table)
            .execute(&mut *conn)
            .await?;
        diesel::delete(assets::table).execute(&mut *conn).await?;
        diesel::delete(folders::table).execute(&mut *conn).await?;
        Ok(())
    })
    .await
    .map_err(|error| error.0)?;
    Ok(())
}

#[async_trait]
impl AccountStore for SqliteCatalog {
    async fn get_current_account(&self) -> Result<Option<Account>, CatalogError> {
        get_current_account(self).await
    }

    async fn get_account(&self, base_url: &str) -> Result<Option<Account>, CatalogError> {
        get_account(self, base_url).await
    }

    async fn save_account(&self, value: &Account) -> Result<(), CatalogError> {
        save_account(self, value).await
    }

    async fn reset_catalog(&self) -> Result<(), CatalogError> {
        reset_catalog(self).await
    }
}
