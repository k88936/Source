//! The subscription protocol under load: many connections, overlapping windows,
//! and writers inserting, moving and deleting rows while the connections
//! subscribe, resubscribe and read.
//!
//! The view is a stand-in for the catalog: a table of rows, each inside a window
//! when its value falls in `[from, to)`. That is enough to exercise every part
//! of the contract - a snapshot, a delta that enters, leaves or stays, a commit
//! that touches one row twice, a query set dropped and taken again, one query
//! answered once for several connections - without a database or a transport.
//!
//! Run with `cargo run -p source-subscription --example echo`.

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::future::Future;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use source_subscription::{
    Change, ClientMessage, ClientReader, ClientWriter, Delta, Query, Row, RowOp, ServerMessage,
    SubscriptionManager, Value, View,
};
use tokio::sync::{mpsc, watch};

/// The table the database holds. Enough rows that windows overlap and a row
/// moves between them as its value changes.
const ROWS: u32 = 600;
/// How many connections read at once.
const PAGES: u32 = 8;
/// How many writers commit at once.
const WRITERS: u32 = 4;
/// How many commits each writer makes.
const COMMITS: u32 = 300;
/// How many distinct windows pages ask for.
///
/// Shared on purpose: the manager answers one query once and sends that answer
/// to every connection holding it.
const WINDOWS: u32 = 4;
/// The window every page holds for the whole run, and checks itself against at
/// the end.
const FIXED: Window = Window { from: 200, to: 800 };
/// The table a change is reported for.
const TABLE: &str = "rows";

/// One interval of the table: the rows whose value is in `[from, to)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Window {
    from: i64,
    to: i64,
}

impl Window {
    fn contains(self, value: i64) -> bool {
        self.from <= value && value < self.to
    }

    /// The key the pages index their copies by.
    fn key(self) -> (i64, i64) {
        (self.from, self.to)
    }
}

/// One row of the table, as a subscriber sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LedgerRow {
    id: String,
    n: i64,
}

impl Row for LedgerRow {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// The window `index` points at.
fn window(index: u32) -> Window {
    let from = i64::from(index) * 180;
    Window {
        from,
        to: from + 240,
    }
}

/// A deterministic generator, so a failing run can be replayed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: u32) -> u32 {
        (self.next() % u64::from(bound)) as u32
    }
}

/// The database: rows by id, written under a lock and read for snapshots.
struct Store {
    rows: Mutex<BTreeMap<u32, i64>>,
}

impl Store {
    fn new() -> Self {
        Self {
            rows: Mutex::new(BTreeMap::new()),
        }
    }

    /// Write one row and report it the way the database would: both sides for an
    /// update, only the new side for an insert.
    fn upsert(&self, id: u32, n: i64) -> Change {
        let mut rows = self.rows.lock().unwrap();
        match rows.insert(id, n) {
            Some(old) => Change::record(
                TABLE,
                columns(),
                RowOp::Update,
                row_values(id, old),
                row_values(id, n),
            ),
            None => Change::record(
                TABLE,
                columns(),
                RowOp::Insert,
                Vec::new(),
                row_values(id, n),
            ),
        }
    }

    fn delete(&self, id: u32) -> Option<Change> {
        let mut rows = self.rows.lock().unwrap();
        let old = rows.remove(&id)?;
        Some(Change::record(
            TABLE,
            columns(),
            RowOp::Delete,
            row_values(id, old),
            Vec::new(),
        ))
    }

    fn window(&self, window: Window) -> Vec<LedgerRow> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, n)| window.contains(**n))
            .map(|(id, n)| LedgerRow {
                id: id.to_string(),
                n: *n,
            })
            .collect()
    }
}

fn columns() -> Arc<[&'static str]> {
    Arc::from(["id", "n"])
}

fn row_values(id: u32, n: i64) -> Vec<Value> {
    vec![Value::Text(id.to_string()), Value::Integer(n)]
}

/// The view over the store: one table, one window parameter.
struct Ledger {
    store: Arc<Store>,
}

impl View for Ledger {
    type Query = Window;
    type Row = LedgerRow;

    fn snapshot(
        &self,
        query: &Window,
    ) -> impl Future<Output = Result<Vec<LedgerRow>, String>> + Send {
        let rows = self.store.window(*query);
        async move { Ok(rows) }
    }

    fn delta(
        &self,
        query: &Window,
        changes: &[Change],
    ) -> impl Future<Output = Result<Delta<LedgerRow>, String>> + Send {
        let delta = answer(*query, changes);
        async move { Ok(delta) }
    }
}

/// What a commit did to one window, decided row by row from the change alone.
///
/// Walked backwards so that a row a commit touched twice is answered by the side
/// it ended on, and never lands on both lists.
fn answer(window: Window, changes: &[Change]) -> Delta<LedgerRow> {
    let mut decided: BTreeMap<String, Option<LedgerRow>> = BTreeMap::new();
    for change in changes.iter().rev() {
        if change.table != TABLE {
            continue;
        }
        let Some(id) = change.text("id") else {
            continue;
        };
        if decided.contains_key(id) {
            continue;
        }
        let before = change.old("n").and_then(Value::as_i64);
        let after = change.new("n").and_then(Value::as_i64);
        let ended = match (
            before.filter(|n| window.contains(*n)),
            after.filter(|n| window.contains(*n)),
        ) {
            // A side inside the window is a row the subscriber must have: a row
            // that moved in, one that changed while staying, and one that
            // arrived and left within the same commit.
            (_, Some(n)) => Some(LedgerRow {
                id: id.to_owned(),
                n,
            }),
            // It was inside and is no longer, so the subscriber must drop it.
            (Some(_), None) => None,
            // Neither side is inside, so this query never saw the row.
            (None, None) => continue,
        };
        decided.insert(id.to_owned(), ended);
    }

    let mut delta = Delta::new();
    for (id, ended) in decided {
        match ended {
            Some(row) => delta.inserts.push(row),
            None => delta.deletes.push(id),
        }
    }
    delta
}

/// One connection, and the rows it believes each query set holds.
struct Page {
    out: mpsc::UnboundedSender<ClientMessage<Window>>,
    messages: mpsc::UnboundedReceiver<ServerMessage<Window, LedgerRow>>,
    /// Set -> window -> rows, exactly what the server has told this page so far.
    rows: BTreeMap<u32, BTreeMap<(i64, i64), BTreeMap<String, LedgerRow>>>,
    next_request: u32,
    next_set: u32,
    applied: u64,
    inserts: u64,
    deletes: u64,
}

impl Page {
    fn open(manager: Arc<SubscriptionManager<Ledger>>) -> Self {
        let (out, incoming) = mpsc::unbounded_channel();
        let (outgoing, messages) = mpsc::unbounded_channel();
        tokio::spawn(manager.run(ChanReader { incoming }, ChanWriter { outgoing }));
        Self {
            out,
            messages,
            rows: BTreeMap::new(),
            next_request: 0,
            next_set: 1,
            applied: 0,
            inserts: 0,
            deletes: 0,
        }
    }

    fn request(&mut self) -> u32 {
        self.next_request += 1;
        self.next_request
    }

    /// Ask for a set of windows. The answer arrives on the stream, in order with
    /// every commit already queued for it.
    fn subscribe(&mut self, queries: Vec<Window>) -> u32 {
        let set = self.next_set;
        self.next_set += 1;
        let request = self.request();
        let _ = self.out.send(ClientMessage::Subscribe {
            request_id: request,
            query_set_id: set,
            queries,
        });
        set
    }

    fn unsubscribe(&mut self, set: u32) {
        let request = self.request();
        let _ = self.out.send(ClientMessage::Unsubscribe {
            request_id: request,
            query_set_id: set,
        });
    }

    /// Apply one message to the copy, keeping only what the wire guarantees.
    fn apply(&mut self, message: ServerMessage<Window, LedgerRow>) {
        self.applied += 1;
        match message {
            ServerMessage::SubscribeApplied {
                query_set_id,
                queries,
                ..
            } => {
                let mut windows = BTreeMap::new();
                for pair in queries {
                    let rows = pair.rows.into_iter().map(|row| (row.id.clone(), row));
                    windows.insert(pair.query.key(), rows.collect());
                }
                self.rows.insert(query_set_id, windows);
            }
            ServerMessage::UnsubscribeApplied { query_set_id, .. } => {
                self.rows.remove(&query_set_id);
            }
            ServerMessage::TransactionUpdate { query_sets } => {
                for update in query_sets {
                    let held = self
                        .rows
                        .get_mut(&update.query_set_id)
                        .unwrap_or_else(|| panic!("update for a set this page never subscribed"));
                    for change in update.queries {
                        let rows = held.get_mut(&change.query.key()).unwrap_or_else(|| {
                            panic!("update for a window this set does not hold")
                        });
                        // A delta that changes nothing is never sent: the view
                        // says so with an empty answer and the manager drops it.
                        assert!(
                            !(change.inserts.is_empty() && change.deletes.is_empty()),
                            "an empty delta reached the wire"
                        );
                        for id in &change.deletes {
                            assert!(
                                !change.inserts.iter().any(|row| &row.id == id),
                                "one row arrived on both sides of a commit"
                            );
                        }
                        self.inserts += change.inserts.len() as u64;
                        self.deletes += change.deletes.len() as u64;
                        for row in change.inserts {
                            rows.insert(row.id.clone(), row);
                        }
                        for id in change.deletes {
                            rows.remove(&id);
                        }
                    }
                }
            }
            ServerMessage::SubscriptionError { error, .. } => {
                panic!("the view failed a query: {error}")
            }
        }
    }

    /// Compare the copy of `set`'s `window` with a snapshot taken now.
    ///
    /// The snapshot is answered while the manager holds the commit lock, and its
    /// answer is queued behind every commit already queued for this connection,
    /// so the two describe the same state. A page that lost a delta, applied one
    /// twice, or read one out of order fails here.
    async fn verify(&mut self, set: u32, window: Window) -> BTreeMap<String, LedgerRow> {
        let probe = self.subscribe(vec![window]);
        loop {
            let Some(message) = self.messages.recv().await else {
                panic!("the connection closed before the snapshot arrived")
            };
            let fresh = match &message {
                ServerMessage::SubscribeApplied {
                    query_set_id,
                    queries,
                    ..
                } if *query_set_id == probe => {
                    let pair = queries
                        .iter()
                        .find(|pair| pair.query == window)
                        .expect("the snapshot answers the window that was asked for");
                    Some(
                        pair.rows
                            .iter()
                            .map(|row| (row.id.clone(), row.clone()))
                            .collect::<BTreeMap<_, _>>(),
                    )
                }
                _ => None,
            };
            self.apply(message);
            if let Some(fresh) = fresh {
                let held = self
                    .rows
                    .get(&set)
                    .and_then(|windows| windows.get(&window.key()))
                    .expect("the page holds the window it is checking");
                assert_eq!(
                    fresh.len(),
                    held.len(),
                    "the copy of {window:?} has the wrong number of rows"
                );
                for (id, row) in &fresh {
                    assert_eq!(
                        held.get(id),
                        Some(row),
                        "the copy of {window:?} disagrees about row {id}"
                    );
                }
                return fresh;
            }
        }
    }
}

/// The inbound half: a channel is a complete transport.
struct ChanReader<Q> {
    incoming: mpsc::UnboundedReceiver<ClientMessage<Q>>,
}

impl<Q: Query> ClientReader<Q> for ChanReader<Q> {
    type Error = Infallible;

    fn recv(
        &mut self,
    ) -> impl Future<Output = Result<Option<ClientMessage<Q>>, Infallible>> + Send {
        async move { Ok(self.incoming.recv().await) }
    }
}

/// The outbound half.
struct ChanWriter<Q, R> {
    outgoing: mpsc::UnboundedSender<ServerMessage<Q, R>>,
}

impl<Q: Query, R: Row> ClientWriter<Q, R> for ChanWriter<Q, R> {
    type Error = Infallible;

    fn send(
        &mut self,
        message: ServerMessage<Q, R>,
    ) -> impl Future<Output = Result<(), Infallible>> + Send {
        async move {
            let _ = self.outgoing.send(message);
            Ok(())
        }
    }
}

/// Read one page: hold a set, resubscribe while commits stream in, then check
/// the copy against a snapshot once the writers are done.
async fn read(
    manager: Arc<SubscriptionManager<Ledger>>,
    seed: u64,
    mut stop: watch::Receiver<bool>,
) -> (Page, BTreeMap<String, LedgerRow>) {
    let mut rng = Rng::new(0xA11CE + seed * 7919);
    let mut page = Page::open(manager);
    let mut held = page.subscribe(vec![FIXED, window(rng.below(WINDOWS))]);
    let mut churned = 0;
    loop {
        tokio::select! {
            incoming = page.messages.recv() => match incoming {
                Some(message) => {
                    page.apply(message);
                    churned += 1;
                    // Drop the set and take another one, while commits keep
                    // arriving: a reload is exactly this, and it must not leave
                    // the page with rows from two different states.
                    if churned % 40 == 0 {
                        page.unsubscribe(held);
                        held = page.subscribe(vec![FIXED, window(rng.below(WINDOWS))]);
                    }
                }
                None => panic!("the connection closed while the run was going"),
            },
            _ = stop.changed() => break,
        }
    }
    let snapshot = page.verify(held, FIXED).await;
    (page, snapshot)
}

/// Write `commits` commits over the ids this writer owns.
///
/// Several writers commit at the same time, and one batch can touch one row more
/// than once, so the manager sees the interleaving the real observe queue sees.
async fn write(
    store: Arc<Store>,
    manager: Arc<SubscriptionManager<Ledger>>,
    writer: u32,
    commits: u32,
) {
    let mut rng = Rng::new(0x51ED + u64::from(writer) * 104729);
    let span = ROWS / WRITERS;
    let first = writer * span;
    for _ in 0..commits {
        let batch = 1 + rng.below(6);
        let mut changes = Vec::with_capacity(batch as usize);
        for _ in 0..batch {
            let id = first + rng.below(span);
            if rng.below(100) < 30 {
                if let Some(change) = store.delete(id) {
                    changes.push(change);
                }
            } else {
                changes.push(store.upsert(id, i64::from(rng.below(1000))));
            }
        }
        manager.apply(&changes).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let store = Arc::new(Store::new());
    let manager = Arc::new(SubscriptionManager::new(Arc::new(Ledger {
        store: Arc::clone(&store),
    })));
    let (stop, _) = watch::channel(false);

    let reads: Vec<_> = (0..PAGES)
        .map(|seed| {
            tokio::spawn(read(
                Arc::clone(&manager),
                u64::from(seed),
                stop.subscribe(),
            ))
        })
        .collect();
    let writes: Vec<_> = (0..WRITERS)
        .map(|writer| {
            tokio::spawn(write(
                Arc::clone(&store),
                Arc::clone(&manager),
                writer,
                COMMITS,
            ))
        })
        .collect();

    for writer in writes {
        writer.await.unwrap();
    }
    stop.send(true).unwrap();

    let mut applied = 0;
    let mut inserts = 0;
    let mut deletes = 0;
    // Every snapshot was taken after the last commit, so they all describe the
    // same state - and it is the state the table is in now.
    let truth: BTreeMap<String, LedgerRow> = store
        .window(FIXED)
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect();
    for reading in reads {
        let (page, snapshot) = reading.await.unwrap();
        applied += page.applied;
        inserts += page.inserts;
        deletes += page.deletes;
        assert_eq!(
            snapshot.len(),
            truth.len(),
            "the snapshot of {FIXED:?} disagrees with the table"
        );
        for (id, row) in &truth {
            assert_eq!(
                snapshot.get(id),
                Some(row),
                "the snapshot of {FIXED:?} disagrees about row {id}"
            );
        }
    }

    println!(
        "ok: {PAGES} pages, {WRITERS} writers, {} commits, {applied} messages, {inserts} inserts, \
         {deletes} deletes across {} rows",
        WRITERS * COMMITS,
        truth.len()
    );
}
