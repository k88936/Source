//! Background runs: one row per pass, so a pass can be seen, cancelled, and
//! picked up again after a restart. The contract and the diesel statements
//! behind it.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;

use crate::time::now_ms;

use self::schema::runs;
use super::*;

mod dto;
pub(crate) mod rows;
pub(crate) mod schema;
mod values;

pub use dto::*;

use self::rows::*;
use self::values::*;

/// Keeps two passes started in the same millisecond apart.
static RUN_SEQ: AtomicU64 = AtomicU64::new(0);

#[async_trait]
pub trait RunStore: Send + Sync {
    /// Open a pass under `key`, and say whether this call is the one that
    /// opened it.
    ///
    /// A pass of the same key that is still running is returned as it is with
    /// `false`, so a second trigger never starts a second pass.
    async fn begin_run(&self, key: RunKey, total: u32) -> Result<(Run, bool), AppError>;
    /// Count visited items, and return the run as it now stands.
    async fn advance_run(&self, id: &RunId, ok: u32, failed: u32) -> Result<Run, AppError>;
    /// Close the pass. A pass that is already closed is returned unchanged.
    async fn finish_run(&self, id: &RunId, state: RunState) -> Result<Run, AppError>;
    /// The newest pass under one key, whatever state it is in.
    async fn get_run(&self, key: RunKey) -> Result<Option<Run>, AppError>;
    /// Reads a pass back into its surroundings without going through the key.
    async fn get_run_by_id(&self, id: &RunId) -> Result<Option<Run>, AppError>;
    /// Mark every pass still running as interrupted, and return how many there
    /// were. Called at startup: a process that died cannot still be busy.
    async fn interrupt_runs(&self) -> Result<u32, AppError>;
}

pub(crate) async fn begin_run(
    cat: &SqliteCatalog,
    key: RunKey,
    total: u32,
) -> Result<(Run, bool), AppError> {
    let mut conn = cat.conn().await?;
    if let Some(running) = runs::table
        .filter(runs::key.eq(key.as_str()))
        .filter(runs::state.eq(run_state_to_str(RunState::Running)))
        .select(run_columns())
        .order(runs::started_at.desc())
        .first::<RunRow>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?
    {
        return Ok((running.into(), false));
    }
    let now = now_ms();
    let id = format!(
        "{}-{}-{}",
        key.as_str(),
        now,
        RUN_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    diesel::insert_into(runs::table)
        .values((
            runs::id.eq(id.clone()),
            runs::key.eq(key.as_str()),
            runs::state.eq(run_state_to_str(RunState::Running)),
            runs::total.eq(i64::from(total)),
            runs::started_at.eq(now),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    let row = runs::table
        .filter(runs::id.eq(id))
        .select(run_columns())
        .first::<RunRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok((row.into(), true))
}

pub(crate) async fn advance_run(
    cat: &SqliteCatalog,
    id: &RunId,
    ok: u32,
    failed: u32,
) -> Result<Run, AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(runs::table.filter(runs::id.eq(id.0.as_str())))
        .set((
            runs::done.eq(runs::done + i64::from(ok)),
            runs::failed.eq(runs::failed + i64::from(failed)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    let row = runs::table
        .filter(runs::id.eq(id.0.as_str()))
        .select(run_columns())
        .first::<RunRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(row.into())
}

pub(crate) async fn finish_run(
    cat: &SqliteCatalog,
    id: &RunId,
    state: RunState,
) -> Result<Run, AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(
        runs::table
            .filter(runs::id.eq(id.0.as_str()))
            .filter(runs::state.eq(run_state_to_str(RunState::Running))),
    )
    .set((
        runs::state.eq(run_state_to_str(state)),
        runs::finished_at.eq(Some(now_ms())),
    ))
    .execute(&mut conn)
    .await
    .map_err(db_err)?;
    let row = runs::table
        .filter(runs::id.eq(id.0.as_str()))
        .select(run_columns())
        .first::<RunRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(row.into())
}

pub(crate) async fn get_run(cat: &SqliteCatalog, key: RunKey) -> Result<Option<Run>, AppError> {
    let mut conn = cat.conn().await?;
    runs::table
        .filter(runs::key.eq(key.as_str()))
        .select(run_columns())
        .order(runs::started_at.desc())
        .first::<RunRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(Run::from))
        .map_err(db_err)
}

pub(crate) async fn get_run_by_id(
    cat: &SqliteCatalog,
    id: &RunId,
) -> Result<Option<Run>, AppError> {
    let mut conn = cat.conn().await?;
    runs::table
        .filter(runs::id.eq(id.0.as_str()))
        .select(run_columns())
        .first::<RunRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(Run::from))
        .map_err(db_err)
}

pub(crate) async fn interrupt_runs(cat: &SqliteCatalog) -> Result<u32, AppError> {
    let mut conn = cat.conn().await?;
    let changed =
        diesel::update(runs::table.filter(runs::state.eq(run_state_to_str(RunState::Running))))
            .set((
                runs::state.eq(run_state_to_str(RunState::Interrupted)),
                runs::finished_at.eq(Some(now_ms())),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_err)?;
    Ok(changed as u32)
}

#[async_trait]
impl RunStore for SqliteCatalog {
    async fn begin_run(&self, key: RunKey, total: u32) -> Result<(Run, bool), AppError> {
        begin_run(self, key, total).await
    }

    async fn advance_run(&self, id: &RunId, ok: u32, failed: u32) -> Result<Run, AppError> {
        advance_run(self, id, ok, failed).await
    }

    async fn finish_run(&self, id: &RunId, state: RunState) -> Result<Run, AppError> {
        finish_run(self, id, state).await
    }

    async fn get_run(&self, key: RunKey) -> Result<Option<Run>, AppError> {
        get_run(self, key).await
    }

    async fn get_run_by_id(&self, id: &RunId) -> Result<Option<Run>, AppError> {
        get_run_by_id(self, id).await
    }

    async fn interrupt_runs(&self) -> Result<u32, AppError> {
        interrupt_runs(self).await
    }
}
