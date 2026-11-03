CREATE TABLE upload_session_records (
  key TEXT NOT NULL REFERENCES upload_sessions(key) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL,
  loaded INTEGER NOT NULL,
  etag TEXT,
  PRIMARY KEY (key, chunk_index)
);

-- Upload sessions are keyed by upload task and consist of one session row plus
-- one row per finished chunk, so the rows written by the old single-blob journal
-- can never be resumed. Drop them instead of leaving unreachable rows behind.
DELETE FROM upload_sessions;

UPDATE upload_queue SET state = 'queued' WHERE state = 'hashing';
