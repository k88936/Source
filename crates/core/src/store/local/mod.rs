//! The local (on-device) media library.

mod rows;

use self::rows::*;
use super::schema::local_assets;
use super::*;

pub(crate) async fn save_local_asset(
    cat: &SqliteCatalog,
    asset: &LocalAsset,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::insert_into(local_assets::table)
        .values((
            local_assets::id.eq(asset.id.clone()),
            local_assets::album_id.eq(asset.album_id.clone()),
            local_assets::name.eq(asset.name.clone()),
            local_assets::mime.eq(asset.mime.clone()),
            local_assets::size.eq(asset.size),
            local_assets::date_taken.eq(asset.date_taken),
            local_assets::uri.eq(asset.uri.clone()),
            local_assets::width.eq(asset.width),
            local_assets::height.eq(asset.height),
            local_assets::duration_ms.eq(asset.duration_ms),
        ))
        .on_conflict(local_assets::id)
        .do_update()
        .set((
            local_assets::album_id.eq(excluded(local_assets::album_id)),
            local_assets::name.eq(excluded(local_assets::name)),
            local_assets::mime.eq(excluded(local_assets::mime)),
            local_assets::size.eq(excluded(local_assets::size)),
            local_assets::date_taken.eq(excluded(local_assets::date_taken)),
            local_assets::uri.eq(excluded(local_assets::uri)),
            local_assets::width.eq(excluded(local_assets::width)),
            local_assets::height.eq(excluded(local_assets::height)),
            local_assets::duration_ms.eq(excluded(local_assets::duration_ms)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn local_asset(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<LocalAsset>, AppError> {
    let mut conn = cat.conn().await?;
    local_assets::table
        .filter(local_assets::id.eq(id))
        .select(LocalAssetRow::as_select())
        .first::<LocalAssetRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(LocalAsset::from))
        .map_err(db_err)
}
