//! The selected server and the key/value settings stored next to it: the
//! contract and the diesel statements behind it.

use async_trait::async_trait;

use crate::time::now_ms;

use self::schema::{app_state as state, server};
use super::*;

mod dto;
pub(crate) mod schema;

pub use dto::*;

#[async_trait]
pub trait ServerStore: Send + Sync {
    async fn save_server(&self, server: &StoredServer) -> Result<(), AppError>;
    async fn load_server(&self) -> Result<Option<StoredServer>, AppError>;
    async fn clear_server(&self) -> Result<(), AppError>;

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError>;
    async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError>;
}

pub(crate) async fn save_server(
    cat: &SqliteCatalog,
    server_record: &StoredServer,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::insert_into(server::table)
        .values((
            server::id.eq("1"),
            server::base_url.eq(server_record.base_url.clone()),
            server::user_id.eq(server_record.user_id.clone()),
            server::display_name.eq(server_record.display_name.clone()),
            server::created_at.eq(now_ms()),
        ))
        .on_conflict(server::id)
        .do_update()
        .set((
            server::base_url.eq(excluded(server::base_url)),
            server::user_id.eq(excluded(server::user_id)),
            server::display_name.eq(excluded(server::display_name)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn load_server(cat: &SqliteCatalog) -> Result<Option<StoredServer>, AppError> {
    let mut conn = cat.conn().await?;
    let row = server::table
        .filter(server::id.eq("1"))
        .select((server::base_url, server::user_id, server::display_name))
        .first::<(String, String, Option<String>)>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    Ok(row.map(|(base_url, user_id, display_name)| StoredServer {
        base_url,
        user_id,
        display_name,
    }))
}

pub(crate) async fn clear_server(cat: &SqliteCatalog) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::delete(server::table)
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
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
impl ServerStore for SqliteCatalog {
    async fn save_server(&self, server_record: &StoredServer) -> Result<(), AppError> {
        save_server(self, server_record).await
    }

    async fn load_server(&self) -> Result<Option<StoredServer>, AppError> {
        load_server(self).await
    }

    async fn clear_server(&self) -> Result<(), AppError> {
        clear_server(self).await
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        set_setting(self, key, value).await
    }

    async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        get_setting(self, key).await
    }
}
