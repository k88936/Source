//! One derived set a client can subscribe to, and the difference a commit makes.

use std::future::Future;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::change::Change;

/// What a client names when it subscribes.
///
/// A query is a parameter, not a statement: the views that answer it are fixed
/// at compile time, so a client can only choose among dimensions the application
/// declared.
///
/// It crosses the wire both ways: the client sends it to subscribe, and every
/// row set comes back naming it, so it has to be encodable.
pub trait Query: Clone + PartialEq + Send + Sync + Serialize + DeserializeOwned + 'static {}

impl<T> Query for T where T: Clone + PartialEq + Send + Sync + Serialize + DeserializeOwned + 'static
{}

/// One row of a view.
///
/// The id is what the client's copy is keyed by, so an insert replaces the row
/// with the same id and a delete names it. A row only travels outwards.
pub trait Row: Clone + Send + Sync + Serialize + 'static {
    fn id(&self) -> String;
}

/// What one query's rows changed by in a commit.
#[derive(Debug, Clone)]
pub struct Delta<R> {
    /// Rows to insert, or to replace by [`Row::id`].
    pub inserts: Vec<R>,
    /// Ids to remove.
    pub deletes: Vec<String>,
}

impl<R> Delta<R> {
    pub fn new() -> Self {
        Self {
            inserts: Vec::new(),
            deletes: Vec::new(),
        }
    }

    /// Whether the commit left this query's rows as they were.
    pub fn is_empty(&self) -> bool {
        self.inserts.is_empty() && self.deletes.is_empty()
    }
}

/// One derived set a client can subscribe to.
///
/// Every method here is `Send`: the manager awaits it inside the task of the
/// connection it is answering, and that task also holds the transport.
///
/// Each method exists because only the implementation knows the answer:
///
/// - [`snapshot`](View::snapshot) is the only full payload a subscriber ever
///   gets, so it runs the query.
/// - [`delta`](View::delta) answers what changed for one query, and an empty
///   answer is how a view says a commit is not visible in it. There is no
///   cheaper question to ask first: whether a change matches a query is the
///   same test the delta makes, so a second method could only repeat it.
///
/// Derived columns (a count, a mosaic) cannot be recovered from row changes, so
/// a view reads the touched rows back. The client never sees the full set again.
pub trait View: Send + Sync + 'static {
    type Query: Query;
    type Row: Row;

    /// Every row the query has now.
    fn snapshot(
        &self,
        query: &Self::Query,
    ) -> impl Future<Output = Result<Vec<Self::Row>, String>> + Send;

    /// What the query's rows changed by.
    ///
    /// Answered from the changes alone: a row belongs to a query or it does not,
    /// and that is decided per side, so the view keeps no copy of what it last
    /// sent.
    fn delta(
        &self,
        query: &Self::Query,
        changes: &[Change],
    ) -> impl Future<Output = Result<Delta<Self::Row>, String>> + Send;
}
