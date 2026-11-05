//! Tauri commands, grouped by the part of the app they serve.

use crate::prelude::*;

pub(crate) mod catalog;
pub(crate) mod connection;
pub(crate) mod export;
pub(crate) mod local;
pub(crate) mod uploads;

/// A run sink that forwards a pass to one event name.
pub(crate) fn run_sink(app: &AppHandle, event: &'static str) -> RunSink {
    let app = app.clone();
    Arc::new(move |run: &Run| {
        let _ = app.emit(event, run);
    })
}
