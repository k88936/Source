//! The subscriptions a page holds: one connection per view.
//!
//! The manager that answers a view lives with the service that owns it, so this
//! module is only the edge: it turns the page's command into a manager call and
//! the manager's messages into events. A webview that reloads closes nothing, so
//! connecting a view again replaces its previous connection: the old inbound
//! half ends, the manager drops every query set it held, and the new page
//! subscribes from a fresh snapshot.
//!
//! Nothing here is view specific beyond the two arms of `ViewName`: the manager,
//! the protocol and the channel adapter are the same crate a standalone server
//! would use.

use std::convert::Infallible;
use std::future::Future;

use source_core::app::browse::AssetsQuery;
use source_subscription::{ClientMessage, ClientWriter, Query, Row, ServerMessage};

use crate::prelude::*;

/// Which view a page's command is for.
///
/// The page names it because one connection serves one view: only the page knows
/// which manager a message belongs to, and a manager cannot answer for a view it
/// does not hold.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ViewName {
    Albums,
    Assets,
}

/// The argument of `subscription_connect`: the view whose connection is opening.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionConnectParams {
    pub(crate) view: ViewName,
}

/// The argument of `subscription_send`: one message, addressed to a view.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionSendParams {
    pub(crate) message: Addressed,
}

/// A page's message, addressed to the view whose manager must answer it.
///
/// The tag is the same word [`ViewName`] reads, so a page that names a view and
/// a message that names one are the same declaration.
#[derive(serde::Deserialize)]
#[serde(tag = "view", rename_all = "camelCase")]
pub(crate) enum Addressed {
    Albums(ClientMessage<()>),
    Assets(ClientMessage<AssetsQuery>),
}

/// Start the connection this page talks through for one view, replacing the one
/// a previous load left behind. Its subscriptions are gone with it, so the new
/// page gets a fresh snapshot for everything it asks for.
#[tauri::command]
pub(crate) fn subscription_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    params: SubscriptionConnectParams,
) -> Result<(), String> {
    match params.view {
        ViewName::Albums => {
            let running = state.core.albums.subscriptions().connect(TauriWriter {
                app,
                event: "subscription://albums",
            })?;
            tauri::async_runtime::spawn(running);
        }
        ViewName::Assets => {
            let running = state.core.browse.subscriptions().connect(TauriWriter {
                app,
                event: "subscription://assets",
            })?;
            tauri::async_runtime::spawn(running);
        }
    }
    Ok(())
}

/// Subscribe or unsubscribe: the message goes to the manager that holds the view
/// it names, as the page wrote it.
#[tauri::command]
pub(crate) fn subscription_send(
    state: State<'_, AppState>,
    params: SubscriptionSendParams,
) -> Result<(), String> {
    match params.message {
        Addressed::Albums(message) => state.core.albums.subscriptions().send(message),
        Addressed::Assets(message) => state.core.browse.subscriptions().send(message),
    }
}

/// The manager's messages, as events the page listens to for one view.
///
/// The event name is the only thing that separates the two streams, so this is
/// the whole tauri-specific half of a connection.
struct TauriWriter {
    app: AppHandle,
    event: &'static str,
}

impl<Q: Query, R: Row> ClientWriter<Q, R> for TauriWriter {
    type Error = Infallible;

    fn send(
        &mut self,
        message: ServerMessage<Q, R>,
    ) -> impl Future<Output = Result<(), Infallible>> + Send {
        async move {
            let _ = self.app.emit(self.event, &message);
            Ok(())
        }
    }
}
