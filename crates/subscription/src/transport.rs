//! Where a connection's bytes live, and nothing else.
//!
//! The two halves are separate traits on purpose: the connection loop awaits a
//! receive and a send at the same time, and one object cannot be borrowed
//! mutably by both. Splitting them is also the shape a socket gives after
//! `split()`.
//!
//! Both futures are `Send` because the loop they run in is spawned; a transport
//! is one task per connection, not a local one.

use std::convert::Infallible;
use std::future::Future;

use tokio::sync::mpsc;

use crate::protocol::{ClientMessage, ServerMessage};
use crate::view::{Query, Row};

/// The inbound half of one client connection.
pub trait ClientReader<Q: Query>: Send + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    /// The next message, or `None` when the client is gone.
    fn recv(
        &mut self,
    ) -> impl Future<Output = Result<Option<ClientMessage<Q>>, Self::Error>> + Send;
}

/// A client that is a channel.
///
/// The manager hands a connection's future a reader, and the page holds the
/// sender: this is that reader. A socket adapter replaces it without anything
/// else changing, because the connection only ever sees [`ClientReader`].
pub struct ChannelReader<Q> {
    messages: mpsc::UnboundedReceiver<ClientMessage<Q>>,
}

impl<Q> ChannelReader<Q> {
    pub fn new(messages: mpsc::UnboundedReceiver<ClientMessage<Q>>) -> Self {
        Self { messages }
    }
}

impl<Q: Query> ClientReader<Q> for ChannelReader<Q> {
    type Error = Infallible;

    fn recv(
        &mut self,
    ) -> impl Future<Output = Result<Option<ClientMessage<Q>>, Infallible>> + Send {
        async move { Ok(self.messages.recv().await) }
    }
}

/// The outbound half of one client connection.
pub trait ClientWriter<Q: Query, R: Row>: Send + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Hand one message to the client. An error ends the connection.
    fn send(
        &mut self,
        message: ServerMessage<Q, R>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
