//! One attempt of a job, retried while the work itself says it is worth it.
//!
//! The queue decides what runs and when; this decides how many times one piece
//! of work tries before it counts as failed. What a visit means - rows, pages,
//! events - stays in the use case, and so does the decision whether a given
//! failure can be helped by trying again.

use std::future::Future;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

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

/// The verdict of one visit.
///
/// It carries its own error in both cases, so the caller keeps the value it
/// wants to show: `Retry` means "this failure may not survive another visit",
/// `Failed` means "this one will not".
#[derive(Debug)]
pub enum Step<T, E> {
    Done(T),
    Retry(E),
    Failed(E),
    /// The visit stopped because the job was asked to stop. Not a failure.
    Stopped,
}

/// What the whole sequence of visits ended as.
#[derive(Debug)]
pub enum Attempt<T, E> {
    Done(T),
    Failed(E),
    Stopped,
}

/// Run `work` until it succeeds, fails for good, or the job is stopped.
///
/// The retry policy belongs to `work`: this only performs the wait and honours
/// the stop. It knows no error type of its own and never decides on its behalf.
pub async fn attempt<T, E, F, Fut>(
    retry: Retry,
    cancel: &CancellationToken,
    mut work: F,
) -> Attempt<T, E>
where
    F: FnMut(Try) -> Fut,
    Fut: Future<Output = Step<T, E>>,
{
    let mut number = 0;
    loop {
        if cancel.is_cancelled() {
            return Attempt::Stopped;
        }
        match work(Try { number, retry }).await {
            Step::Done(value) => return Attempt::Done(value),
            Step::Failed(error) => return Attempt::Failed(error),
            Step::Stopped => return Attempt::Stopped,
            Step::Retry(error) => {
                if number >= retry.attempts {
                    return Attempt::Failed(error);
                }
                number += 1;
                tokio::select! {
                    _ = cancel.cancelled() => return Attempt::Stopped,
                    _ = tokio::time::sleep(retry.delay(number)) => {}
                }
            }
        }
    }
}
