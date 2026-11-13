//! What one committed transaction changed, in the shape a view routes on.

use std::sync::Arc;

/// A table a view reads, with the columns it routes on.
///
/// This is the whole read set a view declares. A table that is not listed is
/// invisible to every view, and a column that is not listed is never read out of
/// a row: the observer only pulls the columns it was told about, so a commit
/// that rewrites a row without touching a listed column still reports the row,
/// but carrying just those columns.
#[derive(Debug, Clone, Copy)]
pub struct Table {
    pub name: &'static str,
    pub columns: &'static [&'static str],
}

impl Table {
    pub const fn new(name: &'static str, columns: &'static [&'static str]) -> Self {
        Self { name, columns }
    }
}

/// Which side of a row a [`Change`] carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowOp {
    Insert,
    Update,
    Delete,
}

impl RowOp {
    /// Whether the row exists after the statement.
    pub fn has_new(self) -> bool {
        matches!(self, Self::Insert | Self::Update)
    }

    /// Whether the row existed before the statement.
    pub fn has_old(self) -> bool {
        matches!(self, Self::Update | Self::Delete)
    }
}

/// One column value as the database had it.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }
}

/// One row a committed transaction inserted, updated or deleted.
///
/// An update is a delete plus an insert in the protocol, but the database reports
/// it once with both sides, so a view can answer "which side stopped matching"
/// and "which side started" from the same change.
#[derive(Debug, Clone)]
pub struct Change {
    pub table: &'static str,
    pub op: RowOp,
    pub columns: Arc<[&'static str]>,
    old: Vec<Value>,
    new: Vec<Value>,
}

impl Change {
    /// One reported row. The old side is empty for an insert, the new side for a
    /// delete.
    pub fn record(
        table: &'static str,
        columns: Arc<[&'static str]>,
        op: RowOp,
        old: Vec<Value>,
        new: Vec<Value>,
    ) -> Self {
        Self {
            table,
            op,
            columns,
            old,
            new,
        }
    }

    /// The value `column` had before the statement, if the change carries one.
    pub fn old(&self, column: &str) -> Option<&Value> {
        self.side(&self.old, column)
    }

    /// The value `column` has after the statement, if the change carries one.
    pub fn new(&self, column: &str) -> Option<&Value> {
        self.side(&self.new, column)
    }

    /// The text `column` has on the new side, else on the old side.
    ///
    /// Routing wants the value that identifies the affected row either way: an
    /// insert only has the new side, a delete only the old one.
    pub fn text(&self, column: &str) -> Option<&str> {
        self.new(column)
            .or_else(|| self.old(column))
            .and_then(Value::as_str)
    }

    fn side<'a>(&'a self, values: &'a [Value], column: &str) -> Option<&'a Value> {
        let index = self.columns.iter().position(|name| *name == column)?;
        values.get(index)
    }
}
