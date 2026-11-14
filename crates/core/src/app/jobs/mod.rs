//! The app's jobs, as the queue and the frontend see them.
//!
//! The queue is generic over what a job is: it carries that value to the list
//! untouched and never reads it. This module is where the app says what its
//! jobs are, so a row can be translated without an id lookup.

mod dto;
mod view;

pub use dto::*;
pub use source_jobs::{
    Ctx, JobChange, JobChangeOp, JobId, JobOutcome, JobPriority, JobState, Progress,
};
pub use view::{JobList, JobRow};

/// One row of the app's job list, with the app's own description of the work.
pub type Job = source_jobs::Job<JobKind>;

/// The app's queue: one budget over every kind of work it submits.
pub type Jobs = source_jobs::Jobs<JobKind>;
