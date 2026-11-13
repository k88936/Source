//! The subscriptions of every connected client.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicU32, Ordering};

use tokio::sync::{Mutex, mpsc};

use crate::change::Change;
use crate::protocol::{
    ClientMessage, QueryRows, QuerySetId, QuerySetUpdate, QueryUpdate, ServerMessage,
};
use crate::transport::{ChannelReader, ClientReader, ClientWriter};
use crate::view::{Delta, View};

/// How many messages may wait for one client before its connection is dropped.
///
/// A dropped delta is not recoverable - the client's copy would stay wrong until
/// it subscribed again - so a client that cannot keep up is disconnected instead
/// of losing a message. This is also why the outbound side is a channel and not a
/// `watch`: every value here is a difference, not a whole state.
const OUTBOUND: usize = 16 * 1024;

/// Every client's subscriptions, and the queries they stand for.
pub struct SubscriptionManager<V: View> {
    views: Arc<V>,
    /// Held while a commit is answered and while a subscription is registered.
    /// A snapshot and the registration cannot be split by a commit, so nothing
    /// that arrived between them is lost.
    applying: Mutex<()>,
    connections: Mutex<Connections<V::Query, V::Row>>,
    next_id: AtomicU32,
    /// Where the page hands its messages in.
    ///
    /// A connection has to exist before the future that reads it is polled, so
    /// its sender is a field here and not an argument to [`run`](Self::run).
    /// Replacing it is what a reload is: the old reader ends, its `run`
    /// disconnects, and the query sets it held go with it.
    inbox: StdMutex<Option<mpsc::UnboundedSender<ClientMessage<V::Query>>>>,
}

struct Connection<Q, R> {
    outbound: mpsc::Sender<ServerMessage<Q, R>>,
    query_sets: HashMap<QuerySetId, Vec<Q>>,
}

/// Every live connection, by the id the page's messages carry.
type Connections<Q, R> = HashMap<u32, Connection<Q, R>>;

impl<V: View> SubscriptionManager<V> {
    pub fn new(views: Arc<V>) -> Self {
        Self {
            views,
            applying: Mutex::new(()),
            connections: Mutex::new(HashMap::new()),
            next_id: AtomicU32::new(1),
            inbox: StdMutex::new(None),
        }
    }

    /// Start the page's connection for this view, replacing the previous one.
    ///
    /// The returned future is the connection: whoever owns an executor decides
    /// where it runs. Calling this again drops the previous inbox, so its
    /// `run` sees the channel end and disconnects, and the new page subscribes
    /// from a fresh snapshot instead of inheriting an old copy.
    pub fn connect<W>(
        self: Arc<Self>,
        writer: W,
    ) -> Result<impl Future<Output = ()> + Send + 'static, String>
    where
        W: ClientWriter<V::Query, V::Row>,
    {
        let (inbox, messages) = mpsc::unbounded_channel();
        let mut slot = self
            .inbox
            .lock()
            .map_err(|_| "subscription: lock poisoned".to_owned())?;
        *slot = Some(inbox);
        drop(slot);
        Ok(async move {
            self.run(ChannelReader::new(messages), writer).await;
        })
    }

    /// Hand one message to the page's connection.
    pub fn send(&self, message: ClientMessage<V::Query>) -> Result<(), String> {
        let inbox = self
            .inbox
            .lock()
            .map_err(|_| "subscription: lock poisoned".to_owned())?;
        let inbox = inbox
            .as_ref()
            .ok_or_else(|| "subscription: no connection".to_owned())?;
        inbox
            .send(message)
            .map_err(|_| "subscription: connection closed".to_owned())
    }

    /// One commit, answered as the difference it made to every query a client
    /// holds.
    ///
    /// The changes are already committed: whoever observed them must not call
    /// this for a rolled-back transaction.
    pub async fn apply(&self, changes: &[Change]) {
        if changes.is_empty() {
            return;
        }
        let _applying = self.applying.lock().await;

        // Only queries a client actually holds are recomputed: a query nobody
        // subscribed to has no rows to keep consistent. The views answer for
        // their own parameters, so an album-scoped query ignores another
        // album's writes.
        let mut subscribed: Vec<V::Query> = Vec::new();
        for connection in self.connections.lock().await.values() {
            for queries in connection.query_sets.values() {
                for query in queries {
                    if !subscribed.contains(query) {
                        subscribed.push(query.clone());
                    }
                }
            }
        }

        let mut deltas: Vec<(V::Query, Delta<V::Row>)> = Vec::new();
        for query in subscribed {
            // A view that nothing in this commit lands in answers with an empty
            // delta, so there is nothing to send for it.
            match self.views.delta(&query, changes).await {
                Ok(delta) if delta.is_empty() => {}
                Ok(delta) => deltas.push((query, delta)),
                Err(error) => {
                    // The view cannot say what changed, so every client holding
                    // it has to subscribe again.
                    self.fail(&query, error).await;
                    return;
                }
            }
        }
        if deltas.is_empty() {
            return;
        }

        let mut stalled = Vec::new();
        {
            let connections = self.connections.lock().await;
            for (id, connection) in connections.iter() {
                let mut query_sets = Vec::new();
                for (query_set_id, queries) in &connection.query_sets {
                    let updates: Vec<QueryUpdate<V::Query, V::Row>> = deltas
                        .iter()
                        .filter(|(query, _)| queries.contains(query))
                        .map(|(query, delta)| QueryUpdate {
                            query: query.clone(),
                            inserts: delta.inserts.clone(),
                            deletes: delta.deletes.clone(),
                        })
                        .collect();
                    if !updates.is_empty() {
                        query_sets.push(QuerySetUpdate {
                            query_set_id: *query_set_id,
                            queries: updates,
                        });
                    }
                }
                if query_sets.is_empty() {
                    continue;
                }
                let message = ServerMessage::TransactionUpdate { query_sets };
                if connection.outbound.try_send(message).is_err() {
                    stalled.push(*id);
                }
            }
        }
        for id in stalled {
            self.disconnect(id).await;
        }
    }

    /// Run one connection until its transport ends.
    ///
    /// The future owns both halves: it answers what the reader hands it and
    /// writes what the manager queued for it. When either side ends, every
    /// subscription this connection held is dropped.
    ///
    /// The caller spawns this on its own runtime. A connection is a task, not a
    /// thread, so whoever owns the executor must be the one that decides one;
    /// a transport adapter knows nothing about it.
    pub async fn run<R, W>(self: Arc<Self>, mut reader: R, mut writer: W)
    where
        R: ClientReader<V::Query>,
        W: ClientWriter<V::Query, V::Row>,
    {
        let (outbound, mut inbox) = mpsc::channel(OUTBOUND);
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.attach(id, outbound).await;
        loop {
            tokio::select! {
                incoming = reader.recv() => match incoming {
                    Ok(Some(message)) => self.handle(id, message).await,
                    Ok(None) | Err(_) => break,
                },
                outgoing = inbox.recv() => match outgoing {
                    Some(message) => {
                        if writer.send(message).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                },
            }
        }
        self.disconnect(id).await;
    }

    async fn attach(&self, id: u32, outbound: mpsc::Sender<ServerMessage<V::Query, V::Row>>) {
        self.connections.lock().await.insert(
            id,
            Connection {
                outbound,
                query_sets: HashMap::new(),
            },
        );
    }

    async fn disconnect(&self, id: u32) {
        drop(self.connections.lock().await.remove(&id));
    }

    async fn handle(&self, id: u32, message: ClientMessage<V::Query>) {
        match message {
            ClientMessage::Subscribe {
                request_id,
                query_set_id,
                queries,
            } => self.subscribe(id, request_id, query_set_id, queries).await,
            ClientMessage::Unsubscribe {
                request_id,
                query_set_id,
            } => self.unsubscribe(id, request_id, query_set_id).await,
        }
    }

    async fn subscribe(
        &self,
        id: u32,
        request_id: u32,
        query_set_id: QuerySetId,
        queries: Vec<V::Query>,
    ) {
        let _applying = self.applying.lock().await;
        // Registered before the snapshot runs, so a commit that lands while the
        // query is executing arrives as a delta instead of being missed. A row
        // delivered twice is harmless: the client's copy is keyed by id.
        {
            let mut connections = self.connections.lock().await;
            let Some(connection) = connections.get_mut(&id) else {
                return;
            };
            connection.query_sets.insert(query_set_id, queries.clone());
        }

        let mut answered = Vec::with_capacity(queries.len());
        for query in &queries {
            match self.views.snapshot(query).await {
                Ok(rows) => answered.push(QueryRows {
                    query: query.clone(),
                    rows,
                }),
                Err(error) => {
                    self.fail_set(id, query_set_id, Some(request_id), error)
                        .await;
                    return;
                }
            }
        }
        self.deliver(
            id,
            ServerMessage::SubscribeApplied {
                request_id,
                query_set_id,
                queries: answered,
            },
        )
        .await;
    }

    async fn unsubscribe(&self, id: u32, request_id: u32, query_set_id: QuerySetId) {
        {
            let mut connections = self.connections.lock().await;
            let Some(connection) = connections.get_mut(&id) else {
                return;
            };
            drop(connection.query_sets.remove(&query_set_id));
        }
        self.deliver(
            id,
            ServerMessage::UnsubscribeApplied {
                request_id,
                query_set_id,
            },
        )
        .await;
    }

    /// Drop a query set on every connection that asked `query` for it.
    async fn fail(&self, query: &V::Query, error: String) {
        let mut affected = Vec::new();
        {
            let mut connections = self.connections.lock().await;
            for (id, connection) in connections.iter_mut() {
                let sets: Vec<QuerySetId> = connection
                    .query_sets
                    .iter()
                    .filter(|(_, queries)| queries.contains(query))
                    .map(|(query_set_id, _)| *query_set_id)
                    .collect();
                for query_set_id in sets {
                    drop(connection.query_sets.remove(&query_set_id));
                    affected.push((*id, query_set_id));
                }
            }
        }
        for (id, query_set_id) in affected {
            self.deliver(
                id,
                ServerMessage::SubscriptionError {
                    request_id: None,
                    query_set_id,
                    error: error.clone(),
                },
            )
            .await;
        }
    }

    async fn fail_set(
        &self,
        id: u32,
        query_set_id: QuerySetId,
        request_id: Option<u32>,
        error: String,
    ) {
        {
            let mut connections = self.connections.lock().await;
            let Some(connection) = connections.get_mut(&id) else {
                return;
            };
            drop(connection.query_sets.remove(&query_set_id));
        }
        self.deliver(
            id,
            ServerMessage::SubscriptionError {
                request_id,
                query_set_id,
                error,
            },
        )
        .await;
    }

    async fn deliver(&self, id: u32, message: ServerMessage<V::Query, V::Row>) {
        let full = {
            let connections = self.connections.lock().await;
            let Some(connection) = connections.get(&id) else {
                return;
            };
            connection.outbound.try_send(message).is_err()
        };
        if full {
            self.disconnect(id).await;
        }
    }
}
