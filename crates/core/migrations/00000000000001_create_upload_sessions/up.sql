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
