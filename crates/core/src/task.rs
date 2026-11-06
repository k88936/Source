//! One attempt of a job, retried while the failure looks transient.
//!
//! The queue decides what runs and when; this decides how many times one piece
//! of work tries before it counts as failed. What a visit means - rows, pages,
//! events - stays in the use case.

use std::future::Future;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::error::AppError;

/// How many times one visit is attempted, and the wait before the first retry.
#[derive(Debug, Clone, Copy)]
pub struct Retry {
    /// Automatic retries after the first failure. `0` never retries.
    pub attempts: u32,
    /// The first wait; it doubles with every further retry.
    pub base: Duration,
}

impl Retry {
    pub const NONE: Retry = Retry {
        attempts: 0,
        base: Duration::ZERO,
    };

    /// Wait before retry number `retried` (1-based).
    pub fn delay(&self, retried: u32) -> Duration {
        let factor = 1u32 << retried.saturating_sub(1).min(4);
        self.base * factor
    }
}

impl Default for Retry {
    fn default() -> Self {
        Retry {
            attempts: 3,
            base: Duration::from_secs(3),
        }
    }
}

/// What an attempt is told about itself: its 0-based number, and the budget.
///
/// The budget is handed over so an attempt can tell the retry that follows from
/// the last one, and record a final failure itself.
#[derive(Debug, Clone, Copy)]
pub struct Try {
    pub number: u32,
    pub retry: Retry,
}

/// Run `work` until it succeeds, fails for good, or the job is stopped.
///
/// Only a retryable failure is tried again, and a stop cuts the wait short
/// instead of counting as one more attempt.
pub async fn attempt<T, F, Fut>(
    retry: Retry,
    cancel: &CancellationToken,
    mut work: F,
) -> Result<T, AppError>
where
    F: FnMut(Try) -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let mut number = 0;
    loop {
        if cancel.is_cancelled() {
            return Err(AppError::cancelled());
        }
        let error = match work(Try { number, retry }).await {
            Ok(value) => return Ok(value),
            Err(error) => error,
        };
        if !error.retryable || number >= retry.attempts {
            return Err(error);
        }
        number += 1;
        tokio::select! {
            _ = cancel.cancelled() => return Err(AppError::cancelled()),
            _ = tokio::time::sleep(retry.delay(number)) => {}
        }
    }
}
