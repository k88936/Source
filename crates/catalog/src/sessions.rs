//! Upload journals: one header row per session plus one row per finished chunk.

use super::*;
use crate::schema::upload_session_records as records;
use crate::schema::upload_sessions as sessions;
use crate::values::*;

pub(crate) async fn create_session(
    cat: &SqliteCatalog,
    key: &str,
    session: &StoredSession,
) -> Result<(), AppError> {
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
                sessions::snapshot.eq(session.bytes.clone()),
                sessions::version.eq(session.version as i32),
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
    index: u32,
    record: &[u8],
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::insert_into(records::table)
        .values((
            records::key.eq(key),
            records::chunk_index.eq(index as i32),
            records::record.eq(record),
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
) -> Result<Option<StoredJournal>, AppError> {
    let mut conn = cat.conn().await?;
    let header = sessions::table
        .filter(sessions::key.eq(key))
        .select((sessions::snapshot, sessions::version, sessions::expires_at))
        .first::<(Vec<u8>, i32, i64)>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    let Some((bytes, version, expires_at)) = header else {
        return Ok(None);
    };
    let rows = records::table
        .filter(records::key.eq(key))
        .select((records::chunk_index, records::record))
        .order(records::chunk_index.asc())
        .load::<(i32, Vec<u8>)>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(Some(StoredJournal {
        session: StoredSession {
            bytes,
            version: version.max(0) as u32,
            expires_at,
        },
        records: rows
            .into_iter()
            .map(|(index, bytes)| StoredRecord {
                index: index.max(0) as u32,
                bytes,
            })
            .collect(),
    }))
}

pub(crate) async fn remove_session(cat: &SqliteCatalog, key: &str) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::delete(sessions::table.filter(sessions::key.eq(key)))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}
