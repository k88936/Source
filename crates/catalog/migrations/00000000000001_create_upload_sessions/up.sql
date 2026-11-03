CREATE TABLE upload_sessions (
  key TEXT PRIMARY KEY,
  snapshot BLOB NOT NULL,
  version INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX upload_sessions_expires ON upload_sessions(expires_at);
