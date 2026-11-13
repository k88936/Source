//! The `assets` table, generated from [`crate::sqlite::MIGRATIONS`].

use diesel::internal::table_macro::StaticQueryFragment;
use diesel::query_source::Column;
use source_subscription::Table;

/// The album an asset belongs to: the column a change of this family routes on.
pub(crate) const ASSET_FOLDER_ID: &str = assets::folder_id::NAME;

/// The key a client's copy of a row is held by, and the id a change names it by.
pub(crate) const ASSET_ID: &str = assets::id::NAME;

/// The column the timeline orders by, and the one a window's anchors point at:
/// a row leaves or enters a window when this pair crosses an anchor.
pub(crate) const ASSET_TAKEN_AT: &str = assets::taken_at::NAME;

/// Only images and videos appear in a window, so a row that stops being one
/// leaves it like any other row.
pub(crate) const ASSET_MEDIA_TYPE: &str = assets::media_type::NAME;

/// What the asset window reads out of this family: the key a client's copy is
/// held by, the album that scopes it, the sort key that places it and the kind
/// that decides whether it belongs in a timeline at all.
pub(crate) const ASSETS: Table = Table::new(
    <assets::table as StaticQueryFragment>::STATIC_COMPONENT.0,
    &[ASSET_ID, ASSET_FOLDER_ID, ASSET_TAKEN_AT, ASSET_MEDIA_TYPE],
);

diesel::table! {
    assets (id) {
        id -> Text,
        folder_id -> Text,
        uri -> Text,
        name -> Text,
        size -> BigInt,
        created_at -> Nullable<BigInt>,
        updated_at -> BigInt,
        remote_updated_at -> BigInt,
        taken_at -> BigInt,
        taken_at_source -> Text,
        media_type -> Text,
        mime -> Nullable<Text>,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        duration_ms -> Nullable<BigInt>,
        favorite -> Bool,
        exif_json -> Nullable<Text>,
        exif_parsed -> Bool,
        source -> Text,
        checksum -> Nullable<Text>,
        thumb_hash -> Nullable<Text>,
    }
}
