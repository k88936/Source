//! The `account` table, generated from [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    account (base_url) {
        base_url -> Text,
        user -> Nullable<Text>,
        site_name -> Nullable<Text>,
        created_at -> BigInt,
    }
}
