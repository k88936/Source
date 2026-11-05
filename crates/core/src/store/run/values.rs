//! String encodings for the run columns.

use super::dto::RunState;

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
