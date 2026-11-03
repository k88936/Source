//! Upload journals: one header row per session plus one row per finished chunk.
//!
//! The table is shared with nothing else: only the uploader writes here, and
//! only through [`UploadSessionPort`].

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel_async::{AsyncConnection, RunQueryDsl};

use super::*;
use crate::sqlite::{Conn, SqliteCatalog, TxError, db_err};
use crate::store::schema::upload_session_records as records;
use crate::store::schema::upload_sessions as sessions;
use crate::time::now_ms;

pub(crate) async fn create_session(
    cat: &SqliteCatalog,
    key: &str,
    session: &UploadSession,
) -> Result<(), AppError> {
    let snapshot = JournalSession::new(session)
        .encode()
        .ok_or_else(|| AppError::internal("无法序列化上传会话"))?;
    let mut conn = cat.conn().await?;
    let now = now_ms();
    conn.transaction::<(), TxError, _>(async |conn| {
        // A header always describes a fresh session, so the chunk rows of the
        // session it replaces can never belong to it.
        diesel::delete(records::table.filter(records::key.eq(key)))
            .execute(&mut *conn)
            .await?;
        diesel::insert_into(sessions::table)
            .values((
                sessions::key.eq(key),
                sessions::snapshot.eq(snapshot.clone()),
                sessions::version.eq(JOURNAL_VERSION as i32),
                sessions::expires_at.eq(session.expires_at),
                sessions::updated_at.eq(now),
            ))
            .on_conflict(sessions::key)
            .do_update()
            .set((
                sessions::snapshot.eq(excluded(sessions::snapshot)),
                sessions::version.eq(excluded(sessions::version)),
                sessions::expires_at.eq(excluded(sessions::expires_at)),
                sessions::updated_at.eq(excluded(sessions::updated_at)),
            ))
            .execute(&mut *conn)
            .await?;
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
    let bytes = serde_json::to_vec(record)
        .map_err(|error| AppError::internal(format!("无法序列化上传分片记录: {error}")))?;
    let mut conn = cat.conn().await?;
    diesel::insert_into(records::table)
        .values((
            records::key.eq(key),
            records::chunk_index.eq(record.index as i32),
            records::record.eq(bytes),
        ))
        .on_conflict((records::key, records::chunk_index))
        .do_update()
        .set(records::record.eq(excluded(records::record)))
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
    let header = sessions::table
        .filter(sessions::key.eq(key))
        .select((sessions::snapshot, sessions::version))
        .first::<(Vec<u8>, i32)>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    let Some((snapshot, version)) = header else {
        return Ok(None);
    };
    // A header this build cannot interpret can never be resumed, so it is
    // dropped instead of being retried on every attempt.
    if version != JOURNAL_VERSION as i32 {
        delete_session(&mut conn, key).await?;
        return Ok(None);
    }
    let Some(header) = JournalSession::decode(&snapshot) else {
        delete_session(&mut conn, key).await?;
        return Ok(None);
    };
    let rows = records::table
        .filter(records::key.eq(key))
        .select(records::record)
        .order(records::chunk_index.asc())
        .load::<Vec<u8>>(&mut conn)
        .await
        .map_err(db_err)?;
    // Chunks that no longer parse only cost a re-upload of that chunk.
    let records = rows
        .iter()
        .filter_map(|bytes| serde_json::from_slice::<ChunkProgress>(bytes).ok())
        .collect();
    Ok(Some((header.session(), records)))
}

pub(crate) async fn remove_session(cat: &SqliteCatalog, key: &str) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    delete_session(&mut conn, key).await
}

async fn delete_session(conn: &mut Conn, key: &str) -> Result<(), AppError> {
    // The chunk rows go with the header: `upload_session_records` cascades on
    // delete, so a resumed session can never read a header that is gone.
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
