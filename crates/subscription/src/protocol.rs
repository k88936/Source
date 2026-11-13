//! The messages a subscriber and the manager exchange.
//!
//! The framing and encoding belong to the transport; these types are only the
//! body. Every message names the query set it is about, because one connection
//! can hold several subscriptions and a client picks the id.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which of a connection's subscriptions a message is about.
pub type QuerySetId = u32;

/// What a subscriber sends.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(
    export,
    export_to = "../../../../packages/subscription-client/src/bindings/",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ClientMessage<Q> {
    /// Ask for a query set. The answer is one [`ServerMessage::SubscribeApplied`]
    /// holding every row.
    Subscribe {
        request_id: u32,
        query_set_id: QuerySetId,
        queries: Vec<Q>,
    },
    /// Drop a query set. Its rows stop arriving.
    Unsubscribe {
        request_id: u32,
        query_set_id: QuerySetId,
    },
}

/// What the manager sends.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(
    export,
    export_to = "../../../../packages/subscription-client/src/bindings/",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ServerMessage<Q, R> {
    /// The first and only full payload: every row of every query in the set.
    SubscribeApplied {
        request_id: u32,
        query_set_id: QuerySetId,
        queries: Vec<QueryRows<Q, R>>,
    },
    /// The query set is gone; the client can forget its rows.
    UnsubscribeApplied {
        request_id: u32,
        query_set_id: QuerySetId,
    },
    /// One commit, in the order it was committed. A query set with nothing to
    /// report is absent.
    TransactionUpdate {
        query_sets: Vec<QuerySetUpdate<Q, R>>,
    },
    /// A query set could not be answered. Its rows on the client are stale until
    /// it subscribes again.
    SubscriptionError {
        request_id: Option<u32>,
        query_set_id: QuerySetId,
        error: String,
    },
}

/// Every row one query has, at the moment the subscription was applied.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../../packages/subscription-client/src/bindings/"
)]
pub struct QueryRows<Q, R> {
    pub query: Q,
    pub rows: Vec<R>,
}

/// What one commit did to one query set.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../../packages/subscription-client/src/bindings/"
)]
pub struct QuerySetUpdate<Q, R> {
    pub query_set_id: QuerySetId,
    pub queries: Vec<QueryUpdate<Q, R>>,
}

/// What one commit did to one query: rows to insert or replace, ids to remove.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../../packages/subscription-client/src/bindings/"
)]
pub struct QueryUpdate<Q, R> {
    pub query: Q,
    pub inserts: Vec<R>,
    pub deletes: Vec<String>,
}
