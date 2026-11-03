CREATE TABLE account (
  base_url TEXT PRIMARY KEY,
  user TEXT,
  site_name TEXT,
  library_root TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE folders (
  id TEXT PRIMARY KEY,
  parent_id TEXT,
  name TEXT NOT NULL,
  uri TEXT NOT NULL DEFAULT '',
  path TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  indexed_at INTEGER,
  is_album INTEGER NOT NULL DEFAULT 0,
  cover_asset_id TEXT,
  description TEXT
);
CREATE INDEX folders_path ON folders(path);
CREATE INDEX folders_parent ON folders(parent_id);

CREATE TABLE assets (
  id TEXT PRIMARY KEY,
  folder_id TEXT NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
  uri TEXT NOT NULL DEFAULT '',
  name TEXT NOT NULL,
  size INTEGER NOT NULL,
  created_at INTEGER,
  updated_at INTEGER NOT NULL,
  remote_updated_at INTEGER NOT NULL DEFAULT 0,
  taken_at INTEGER NOT NULL,
  taken_at_source TEXT NOT NULL,
  media_type TEXT NOT NULL,
  mime TEXT,
  width INTEGER,
  height INTEGER,
  duration_ms INTEGER,
  favorite INTEGER NOT NULL DEFAULT 0,
  exif_json TEXT,
  exif_parsed INTEGER NOT NULL DEFAULT 0,
  source TEXT NOT NULL DEFAULT 'remote',
  checksum TEXT,
  thumb_hash TEXT
);
CREATE INDEX assets_timeline ON assets(taken_at DESC, id DESC);
CREATE INDEX assets_folder ON assets(folder_id, taken_at DESC);
CREATE INDEX assets_name ON assets(name);

CREATE TABLE sync_state (
  folder_id TEXT PRIMARY KEY,
  last_listed_at INTEGER,
  last_page_token TEXT,
  remote_updated_at INTEGER,
  status TEXT NOT NULL,
  last_error TEXT
);

CREATE TABLE local_albums (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  asset_count INTEGER
);

CREATE TABLE local_assets (
  id TEXT PRIMARY KEY,
  album_id TEXT,
  name TEXT NOT NULL,
  mime TEXT,
  size INTEGER,
  date_taken INTEGER,
  uri TEXT NOT NULL,
  width INTEGER,
  height INTEGER,
  duration_ms INTEGER,
  checksum TEXT,
  uploaded_asset_id TEXT
);

CREATE TABLE upload_queue (
  id TEXT PRIMARY KEY,
  local_asset_id TEXT NOT NULL,
  dest_uri TEXT NOT NULL,
  dest_name TEXT,
  name TEXT NOT NULL DEFAULT '',
  state TEXT NOT NULL,
  uploaded INTEGER NOT NULL DEFAULT 0,
  total INTEGER NOT NULL DEFAULT 0,
  speed INTEGER NOT NULL DEFAULT 0,
  eta_seconds INTEGER,
  attempts INTEGER NOT NULL DEFAULT 0,
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX upload_queue_state ON upload_queue(state, created_at);

CREATE TABLE upload_sessions (
  key TEXT PRIMARY KEY,
  file_size INTEGER NOT NULL,
  chunk_size INTEGER NOT NULL,
  session_id TEXT NOT NULL,
  credential TEXT NOT NULL,
  upload_id TEXT NOT NULL,
  uri TEXT NOT NULL,
  callback_secret TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  complete_url TEXT,
  upload_policy TEXT,
  mime_type TEXT,
  policy_type TEXT,
  relay INTEGER,
  chunk_concurrency INTEGER,
  encrypt_key_plain_text TEXT,
  encrypt_iv TEXT,
  updated_at INTEGER NOT NULL
);
CREATE INDEX upload_sessions_expires ON upload_sessions(expires_at);

CREATE TABLE upload_session_urls (
  key TEXT NOT NULL REFERENCES upload_sessions(key) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL,
  url TEXT NOT NULL,
  PRIMARY KEY (key, chunk_index)
);

CREATE TABLE upload_session_records (
  key TEXT NOT NULL REFERENCES upload_sessions(key) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL,
  loaded INTEGER NOT NULL,
  etag TEXT,
  PRIMARY KEY (key, chunk_index)
);

CREATE TABLE app_state (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
