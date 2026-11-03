-- Reverting drops every indexed folder, so the next index pass rebuilds it.
PRAGMA foreign_keys = OFF;
DROP TABLE upload_session_records;
DROP TABLE upload_session_urls;
DROP TABLE upload_sessions;
DROP TABLE upload_queue;
DROP TABLE local_assets;
DROP TABLE local_albums;
DROP TABLE assets;
DROP TABLE sync_state;
DROP TABLE folders;
DROP TABLE app_state;
DROP TABLE account;
PRAGMA foreign_keys = ON;
