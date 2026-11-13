//! Subscription views: what a client watches, and the rows that changed since.
//!
//! A subscriber names a query once and gets one full snapshot back; every
//! commit after that is delivered as the difference it made to that query. The
//! pieces are SpacetimeDB's, with the same names: a [`View`] is one derived set
//! a client can subscribe to, a [`Query`] is its parameter, [`SubscribeApplied`]
//! answers the first subscribe and [`TransactionUpdate`] answers every commit.
//!
//! Nothing here knows about a database, a transport or an application. The
//! changed rows come from whoever owns the write path, as [`Change`] values; the
//! messages go out through a [`ClientWriter`] and come in through a
//! [`ClientReader`]. A pair of channels is therefore a complete transport, and a
//! socket adapter only has to implement the same two traits.
//!
//! [`SubscribeApplied`]: ServerMessage::SubscribeApplied
//! [`TransactionUpdate`]: ServerMessage::TransactionUpdate

#![allow(async_fn_in_trait)]

mod change;
mod manager;
mod protocol;
mod transport;
mod view;

pub use change::{Change, RowOp, Table, Value};
pub use manager::SubscriptionManager;
pub use protocol::{
    ClientMessage, QueryRows, QuerySetId, QuerySetUpdate, QueryUpdate, ServerMessage,
};
pub use transport::{ChannelReader, ClientReader, ClientWriter};
pub use view::{Delta, Query, Row, View};
