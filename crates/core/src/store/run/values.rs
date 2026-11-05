//! String encodings for the run columns.

use super::dto::{RunKind, RunState};

pub(crate) fn run_kind_to_str(value: RunKind) -> &'static str {
    match value {
        RunKind::Index => "index",
        RunKind::Sync => "sync",
    }
}

pub(crate) fn run_kind_from_str(value: &str) -> RunKind {
    match value {
        "sync" => RunKind::Sync,
        _ => RunKind::Index,
    }
}

pub(crate) fn run_state_to_str(value: RunState) -> &'static str {
    match value {
        RunState::Running => "running",
        RunState::Done => "done",
        RunState::Failed => "failed",
        RunState::Cancelled => "cancelled",
        RunState::Interrupted => "interrupted",
    }
}

pub(crate) fn run_state_from_str(value: &str) -> RunState {
    match value {
        "done" => RunState::Done,
        "failed" => RunState::Failed,
        "cancelled" => RunState::Cancelled,
        "interrupted" => RunState::Interrupted,
        _ => RunState::Running,
    }
}
