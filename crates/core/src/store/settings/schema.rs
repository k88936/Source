//! The `app_state` table, generated from [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    app_state (key) {
        key -> Text,
        value -> Text,
    }
}
