//! The `folders` and `sync_state` tables, generated from
//! [`crate::sqlite::MIGRATIONS`].

use diesel::internal::table_macro::StaticQueryFragment;
use diesel::query_source::Column;
use source_subscription::Table;

/// The column that names the album a row of this family is about.
pub(crate) const FOLDER_ID: &str = folders::id::NAME;
pub(crate) const SYNC_FOLDER_ID: &str = sync_state::folder_id::NAME;

/// What the album view reads out of this family: the album row is its own group,
/// and a listing outcome names the album it is about.
pub(crate) const FOLDERS: Table = Table::new(
    <folders::table as StaticQueryFragment>::STATIC_COMPONENT.0,
    &[FOLDER_ID],
);
pub(crate) const SYNC_STATE: Table = Table::new(
    <sync_state::table as StaticQueryFragment>::STATIC_COMPONENT.0,
    &[SYNC_FOLDER_ID],
);

diesel::table! {
    folders (id) {
        id -> Text,
        name -> Text,
        uri -> Text,
        path -> Text,
        updated_at -> BigInt,
        indexed_at -> Nullable<BigInt>,
        cover_asset_id -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    sync_state (folder_id) {
        folder_id -> Text,
        last_listed_at -> Nullable<BigInt>,
        last_page_token -> Nullable<Text>,
        remote_updated_at -> Nullable<BigInt>,
        status -> Text,
    }
}
