//! The `upload_queue` table, generated from [`crate::sqlite::MIGRATIONS`].

use diesel::internal::table_macro::StaticQueryFragment;
use diesel::query_source::Column;
use source_subscription::Table;

/// The key a client's copy of a queued upload is held by.
pub(crate) const UPLOAD_TASK_ID: &str = upload_queue::id::NAME;

/// What the upload view reads out of this table: the row itself, asked again by
/// id whenever a commit names one.
pub(crate) const UPLOAD_QUEUE: Table = Table::new(
    <upload_queue::table as StaticQueryFragment>::STATIC_COMPONENT.0,
    &[UPLOAD_TASK_ID],
);

diesel::table! {
    upload_queue (id) {
        id -> Text,
        local_asset_id -> Text,
        local_album_id -> Nullable<Text>,
        dest_folder_id -> Text,
        dest_uri -> Text,
        dest_name -> Nullable<Text>,
        name -> Text,
        state -> Text,
        uploaded -> BigInt,
        total -> BigInt,
        speed -> BigInt,
        eta_seconds -> Nullable<BigInt>,
        error -> Nullable<Text>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}
