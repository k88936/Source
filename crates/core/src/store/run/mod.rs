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
    /// Open a pass, and say whether this call is the one that opened it.
    ///
    /// A pass of the same kind that is still running is returned as it is with
    /// `false`, so a second trigger never starts a second pass.
    async fn begin_run(&self, kind: RunKind, total: u32) -> Result<(Run, bool), AppError>;
    /// Count visited items, and return the run as it now stands.
    async fn advance_run(&self, id: &str, ok: u32, failed: u32) -> Result<Run, AppError>;
    /// Close the pass. A pass that is already closed is returned unchanged.
    async fn finish_run(&self, id: &str, state: RunState) -> Result<Run, AppError>;
    /// The newest pass of one kind, whatever state it is in.
    async fn run(&self, kind: RunKind) -> Result<Option<Run>, AppError>;
    /// Mark every pass still running as interrupted, and return how many there
    /// were. Called at startup: a process that died cannot still be busy.
    async fn interrupt_runs(&self) -> Result<u32, AppError>;
}

pub(crate) async fn begin_run(
    cat: &SqliteCatalog,
    kind: RunKind,
    total: u32,
) -> Result<(Run, bool), AppError> {
    let mut conn = cat.conn().await?;
    if let Some(running) = runs::table
        .filter(runs::kind.eq(run_kind_to_str(kind)))
        .filter(runs::state.eq(run_state_to_str(RunState::Running)))
        .select(RunRow::as_select())
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
        run_kind_to_str(kind),
        now,
        RUN_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    diesel::insert_into(runs::table)
        .values((
            runs::id.eq(id.clone()),
            runs::kind.eq(run_kind_to_str(kind)),
            runs::state.eq(run_state_to_str(RunState::Running)),
            runs::total.eq(i64::from(total)),
            runs::started_at.eq(now),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    let row = runs::table
        .filter(runs::id.eq(id))
        .select(RunRow::as_select())
        .first::<RunRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok((row.into(), true))
}

pub(crate) async fn advance_run(
    cat: &SqliteCatalog,
    id: &str,
    ok: u32,
    failed: u32,
) -> Result<Run, AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(runs::table.filter(runs::id.eq(id)))
        .set((
            runs::done.eq(runs::done + i64::from(ok)),
            runs::failed.eq(runs::failed + i64::from(failed)),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    let row = runs::table
        .filter(runs::id.eq(id))
        .select(RunRow::as_select())
        .first::<RunRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(row.into())
}

pub(crate) async fn finish_run(
    cat: &SqliteCatalog,
    id: &str,
    state: RunState,
) -> Result<Run, AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(
        runs::table
            .filter(runs::id.eq(id))
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
        .filter(runs::id.eq(id))
        .select(RunRow::as_select())
        .first::<RunRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(row.into())
}

pub(crate) async fn run(cat: &SqliteCatalog, kind: RunKind) -> Result<Option<Run>, AppError> {
    let mut conn = cat.conn().await?;
    runs::table
        .filter(runs::kind.eq(run_kind_to_str(kind)))
        .select(RunRow::as_select())
        .order(runs::started_at.desc())
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
    async fn begin_run(&self, kind: RunKind, total: u32) -> Result<(Run, bool), AppError> {
        begin_run(self, kind, total).await
    }

    async fn advance_run(&self, id: &str, ok: u32, failed: u32) -> Result<Run, AppError> {
        advance_run(self, id, ok, failed).await
    }

    async fn finish_run(&self, id: &str, state: RunState) -> Result<Run, AppError> {
        finish_run(self, id, state).await
    }

    async fn run(&self, kind: RunKind) -> Result<Option<Run>, AppError> {
        run(self, kind).await
    }

    async fn interrupt_runs(&self) -> Result<u32, AppError> {
        interrupt_runs(self).await
    }
}
