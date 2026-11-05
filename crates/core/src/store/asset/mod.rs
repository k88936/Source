//! Indexed assets and the per-asset flags the UI can toggle: the contract and
//! the diesel statements behind it.

use async_trait::async_trait;

use self::schema::assets;
use super::query::*;
use super::*;

mod dto;
pub(crate) mod rows;
pub(crate) mod schema;
pub(crate) mod values;

pub use dto::*;

use self::rows::*;

#[async_trait]
pub trait AssetStore: Send + Sync {
    async fn upsert_asset(&self, asset: &AssetRecord) -> Result<(), AppError>;
    /// Re-point an asset at another folder and URI after a remote move.
    async fn move_asset(&self, asset_id: &str, folder_id: &str, uri: &str) -> Result<(), AppError>;
    async fn asset(&self, id: &str) -> Result<Option<AssetRecord>, AppError>;
    async fn set_favorite(&self, asset_id: &str, value: bool) -> Result<(), AppError>;
    /// Store the blur-up mosaic for a tile. Written once per asset, lazily.
    async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), AppError>;
}

pub(crate) async fn upsert_asset(cat: &SqliteCatalog, asset: &AssetRecord) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    upsert_asset_row(&mut conn, asset).await
}

pub(crate) async fn asset(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<AssetRecord>, AppError> {
    let mut conn = cat.conn().await?;
    let row = assets::table
        .filter(assets::id.eq(id))
        .select((
            AssetRow::as_select(),
            assets::uri,
            assets::exif_json,
            assets::remote_updated_at,
        ))
        .first::<(AssetRow, String, Option<String>, i64)>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    Ok(row.map(|(row, uri, exif_json, remote_updated_at)| {
        let summary = AssetSummary::from(row);
        AssetRecord {
            taken_at_source: summary.taken_at_source,
            summary,
            uri,
            exif_json,
            remote_updated_at,
        }
    }))
}

pub(crate) async fn set_favorite(
    cat: &SqliteCatalog,
    asset_id: &str,
    value: bool,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
        .set(assets::favorite.eq(value))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn move_asset(
    cat: &SqliteCatalog,
    asset_id: &str,
    folder_id: &str,
    uri: &str,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
        .set((assets::folder_id.eq(folder_id), assets::uri.eq(uri)))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn set_thumb_hash(
    cat: &SqliteCatalog,
    asset_id: &str,
    value: &str,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
        .set(assets::thumb_hash.eq(value))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

#[async_trait]
impl AssetStore for SqliteCatalog {
    async fn upsert_asset(&self, asset_record: &AssetRecord) -> Result<(), AppError> {
        upsert_asset(self, asset_record).await
    }

    async fn move_asset(&self, asset_id: &str, folder_id: &str, uri: &str) -> Result<(), AppError> {
        move_asset(self, asset_id, folder_id, uri).await
    }

    async fn asset(&self, id: &str) -> Result<Option<AssetRecord>, AppError> {
        asset(self, id).await
    }

    async fn set_favorite(&self, asset_id: &str, value: bool) -> Result<(), AppError> {
        set_favorite(self, asset_id, value).await
    }

    async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), AppError> {
        set_thumb_hash(self, asset_id, value).await
    }
}
