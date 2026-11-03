//! Durable storage for resumable upload sessions: the contract and the diesel
//! statements behind it.

use async_trait::async_trait;
use cloudreve_sdk_uploader::{ChunkProgress, UploadSession};

use self::schema::upload_session_records as records;
use self::schema::upload_session_urls as urls;
use self::schema::upload_sessions as sessions;
use super::*;
use crate::time::now_ms;

mod rows;
pub(crate) mod schema;
mod values;

use self::rows::*;
use self::values::*;

/// Durable storage for resumable upload sessions.
///
/// A session is append-only: the session row is written once, then one row per
/// finished chunk. Re-appending a chunk index must replace that row instead of
/// growing the session, so a retried chunk cannot corrupt the replay.
#[async_trait]
pub trait UploadSessionPort: Send + Sync {
    /// Store a new session under `key`, dropping the chunk rows of the session
    /// it replaces.
    async fn create_session(&self, key: &str, session: &UploadSession) -> Result<(), AppError>;
    /// Append one finished chunk, replacing the row of a chunk retried within
    /// the same session.
    async fn append_chunk(&self, key: &str, record: &ChunkProgress) -> Result<(), AppError>;
    /// Return the session and the chunks that can be resumed.
    async fn load_session(
        &self,
        key: &str,
    ) -> Result<Option<(UploadSession, Vec<ChunkProgress>)>, AppError>;
    /// Remove the session row and every chunk recorded under `key`.
    async fn remove_session(&self, key: &str) -> Result<(), AppError>;
}

pub(crate) async fn create_session(
    cat: &SqliteCatalog,
    key: &str,
    session: &UploadSession,
) -> Result<(), AppError> {
    let credential = session.credential();
    let policy = credential.storage_policy.as_ref();
    let encryption = credential.encrypt_metadata.as_ref();
    let mut conn = cat.conn().await?;
    let now = now_ms();
    conn.transaction::<(), TxError, _>(async |conn| {
        // A session row always describes a fresh session, so the chunk rows and
        // the upload URLs of the session it replaces can never belong to it.
        diesel::delete(records::table.filter(records::key.eq(key)))
            .execute(&mut *conn)
            .await?;
        diesel::delete(urls::table.filter(urls::key.eq(key)))
            .execute(&mut *conn)
            .await?;
        diesel::insert_into(sessions::table)
            .values((
                sessions::key.eq(key),
                sessions::file_size.eq(session.file_size() as i64),
                sessions::chunk_size.eq(session.chunk_size() as i64),
                sessions::session_id.eq(credential.session_id.as_str()),
                sessions::credential.eq(credential.credential.as_str()),
                sessions::upload_id.eq(credential.upload_id.as_str()),
                sessions::uri.eq(credential.uri.as_str()),
                sessions::callback_secret.eq(credential.callback_secret.as_str()),
                sessions::expires_at.eq(session.expires_at()),
                sessions::complete_url.eq(credential.complete_url.as_deref()),
                sessions::upload_policy.eq(credential.upload_policy.as_deref()),
                sessions::mime_type.eq(credential.mime_type.as_deref()),
                sessions::policy_type.eq(policy.map(|p| policy_type_to_str(&p.policy_type))),
                sessions::relay.eq(policy.and_then(|p| p.relay)),
                sessions::chunk_concurrency.eq(policy.and_then(|p| p.chunk_concurrency)),
                sessions::encrypt_key_plain_text.eq(encryption.map(|e| e.key_plain_text.as_str())),
                sessions::encrypt_iv.eq(encryption.map(|e| e.iv.as_str())),
                sessions::updated_at.eq(now),
            ))
            .on_conflict(sessions::key)
            .do_update()
            .set((
                sessions::file_size.eq(excluded(sessions::file_size)),
                sessions::chunk_size.eq(excluded(sessions::chunk_size)),
                sessions::session_id.eq(excluded(sessions::session_id)),
                sessions::credential.eq(excluded(sessions::credential)),
                sessions::upload_id.eq(excluded(sessions::upload_id)),
                sessions::uri.eq(excluded(sessions::uri)),
                sessions::callback_secret.eq(excluded(sessions::callback_secret)),
                sessions::expires_at.eq(excluded(sessions::expires_at)),
                sessions::complete_url.eq(excluded(sessions::complete_url)),
                sessions::upload_policy.eq(excluded(sessions::upload_policy)),
                sessions::mime_type.eq(excluded(sessions::mime_type)),
                sessions::policy_type.eq(excluded(sessions::policy_type)),
                sessions::relay.eq(excluded(sessions::relay)),
                sessions::chunk_concurrency.eq(excluded(sessions::chunk_concurrency)),
                sessions::encrypt_key_plain_text.eq(excluded(sessions::encrypt_key_plain_text)),
                sessions::encrypt_iv.eq(excluded(sessions::encrypt_iv)),
                sessions::updated_at.eq(excluded(sessions::updated_at)),
            ))
            .execute(&mut *conn)
            .await?;
        if let Some(session_urls) = credential.upload_urls.as_ref() {
            // One row per chunk: a provider addresses the chunk URL by index, so
            // the order the server sent the URLs in has to survive.
            for (chunk_index, url) in session_urls.iter().enumerate() {
                diesel::insert_into(urls::table)
                    .values((
                        urls::key.eq(key),
                        urls::chunk_index.eq(chunk_index as i32),
                        urls::url.eq(url.as_str()),
                    ))
                    .execute(&mut *conn)
                    .await?;
            }
        }
        Ok(())
    })
    .await
    .map_err(|error| error.0)
}

pub(crate) async fn append_chunk(
    cat: &SqliteCatalog,
    key: &str,
    record: &ChunkProgress,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::insert_into(records::table)
        .values((
            records::key.eq(key),
            records::chunk_index.eq(record.index as i32),
            records::loaded.eq(record.loaded as i64),
            records::etag.eq(record.etag.clone()),
        ))
        .on_conflict((records::key, records::chunk_index))
        .do_update()
        .set((
            records::loaded.eq(excluded(records::loaded)),
            records::etag.eq(excluded(records::etag)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn load_session(
    cat: &SqliteCatalog,
    key: &str,
) -> Result<Option<(UploadSession, Vec<ChunkProgress>)>, AppError> {
    let mut conn = cat.conn().await?;
    let row = sessions::table
        .filter(sessions::key.eq(key))
        .select(UploadSessionRow::as_select())
        .first::<UploadSessionRow>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let upload_urls = urls::table
        .filter(urls::key.eq(key))
        .order(urls::chunk_index.asc())
        .select(urls::url)
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)?;
    let rows = records::table
        .filter(records::key.eq(key))
        .select(UploadSessionRecordRow::as_select())
        .order(records::chunk_index.asc())
        .load::<UploadSessionRecordRow>(&mut conn)
        .await
        .map_err(db_err)?;
    let records = rows.into_iter().map(ChunkProgress::from).collect();
    Ok(Some((row.into_session(upload_urls)?, records)))
}

pub(crate) async fn remove_session(cat: &SqliteCatalog, key: &str) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    delete_session(&mut conn, key).await
}

async fn delete_session(conn: &mut Conn, key: &str) -> Result<(), AppError> {
    // The chunk rows and the upload URLs go with the session row: both tables
    // cascade on delete, so nothing can outlive the session it belongs to.
    diesel::delete(sessions::table.filter(sessions::key.eq(key)))
        .execute(conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

#[async_trait]
impl UploadSessionPort for SqliteCatalog {
    async fn create_session(&self, key: &str, session: &UploadSession) -> Result<(), AppError> {
        create_session(self, key, session).await
    }

    async fn append_chunk(&self, key: &str, record: &ChunkProgress) -> Result<(), AppError> {
        append_chunk(self, key, record).await
    }

    async fn load_session(
        &self,
        key: &str,
    ) -> Result<Option<(UploadSession, Vec<ChunkProgress>)>, AppError> {
        load_session(self, key).await
    }

    async fn remove_session(&self, key: &str) -> Result<(), AppError> {
        remove_session(self, key).await
    }
}
