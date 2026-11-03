//! Key/value settings stored next to the catalog: the contract and the diesel
//! statements behind it.

use async_trait::async_trait;

use self::schema::app_state as state;
use super::*;

pub(crate) mod schema;

#[async_trait]
pub trait SettingsStore: Send + Sync {
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError>;
    async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError>;
}

pub(crate) async fn set_setting(
    cat: &SqliteCatalog,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::insert_into(state::table)
        .values((state::key.eq(key), state::value.eq(value)))
        .on_conflict(state::key)
        .do_update()
        .set(state::value.eq(excluded(state::value)))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn get_setting(
    cat: &SqliteCatalog,
    key: &str,
) -> Result<Option<String>, AppError> {
    let mut conn = cat.conn().await?;
    state::table
        .filter(state::key.eq(key))
        .select(state::value)
        .first::<String>(&mut conn)
        .await
        .optional()
        .map_err(db_err)
}

#[async_trait]
impl SettingsStore for SqliteCatalog {
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        set_setting(self, key, value).await
    }

    async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        get_setting(self, key).await
    }
}
