//! Observing committed catalog writes from SQLite itself.
//!
//! A pooled diesel connection exposes no way to see the statements it runs, so
//! the observation point is SQLite. Three hooks and one trace flag, installed on
//! every connection this process opens:
//!
//! - `preupdate` reports one row per statement, carrying only the declared
//!   columns, and it runs while the write lock is still held. `update_hook`
//!   cannot do this job: for a delete it only names a rowid, and the row that
//!   held the album id is already gone.
//! - `commit` marks the transaction as one that may be published; `rollback`
//!   throws its rows away, because a write that did not happen is not a change.
//! - `SQLITE_TRACE_PROFILE` fires when a statement, including the implicit
//!   commit at its end, has finished, so a batch is only published once a reader
//!   connection can see it. That is what lets the view queries run immediately
//!   instead of racing the writer. The rows are kept per connection, so a reader
//!   ending a statement cannot publish a writer's batch before that writer's own
//!   statement has ended.
//!
//! The hooks are registered through `sqlite3_auto_extension`, which SQLite calls
//! for every connection opened after registration, so the pooled connections and
//! the one that runs migrations are all covered without touching the pool.
//!
//! What is left here is the pointer work SQLite's C interface forces on a
//! callback: four registrations, one row read, and the four values a cell can
//! hold. Anything that can be done with a statement - learning a table's column
//! layout, for instance - is done in [`resolve`], outside the callbacks, with
//! diesel.

use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::ptr;
use std::sync::{Arc, LazyLock, Mutex, Once, OnceLock};

use diesel::sql_types::{Integer, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};
use libsqlite3_sys as ffi;
use source_subscription::{Change, RowOp, Table, Value};
use tokio::sync::mpsc::UnboundedSender;

use super::{CatalogError, db_err};

/// Where a committed transaction goes.
static SINK: OnceLock<UnboundedSender<Vec<Change>>> = OnceLock::new();

/// The rows of the transaction each connection is writing.
///
/// A transaction belongs to the connection that runs it, and only that
/// connection's own `profile` publishes the batch. Keying the state by connection
/// keeps a reader that ends a statement from taking a writer's rows before the
/// writer's commit is visible.
static CHANGES: LazyLock<Mutex<HashMap<usize, Pending>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The columns each declared table routes on, resolved once per table.
///
/// The declaration names columns; a row is read by index. [`resolve`] pairs the
/// two once, after migrations, so no callback ever prepares a statement.
static SCHEMA: OnceLock<HashMap<&'static str, Declared>> = OnceLock::new();

struct Pending {
    changes: Vec<Change>,
    committed: bool,
}

impl Pending {
    const fn new() -> Self {
        Self {
            changes: Vec::new(),
            committed: false,
        }
    }
}

/// A declared table and where its columns sit in a row.
struct Declared {
    name: &'static str,
    columns: Arc<[&'static str]>,
    indexes: Vec<i32>,
}

/// Route every committed transaction to `sink`.
///
/// Must run before the first connection is opened, because a connection keeps
/// the sink it captured at registration.
pub fn observe_commits(sink: UnboundedSender<Vec<Change>>) {
    let _ = SINK.set(sink);
}

/// Install the observer on every connection this process opens.
///
/// Must run before the first connection is opened: `sqlite3_auto_extension` only
/// applies to connections opened after it is registered.
pub(crate) fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // SAFETY: the pointer is a plain `extern "C"` function with the signature
        // SQLite expects, and the callbacks only touch this module's state.
        unsafe { ffi::sqlite3_auto_extension(Some(init)) };
    });
}

unsafe extern "C" fn init(
    db: *mut ffi::sqlite3,
    _error: *mut *mut c_char,
    _api: *const ffi::sqlite3_api_routines,
) -> c_int {
    // A closed connection's address can be reused, so the state keyed by this
    // one starts empty.
    if let Ok(mut batches) = CHANGES.lock() {
        batches.remove(&(db as usize));
    }
    // SAFETY: `db` is the connection SQLite just opened, and it outlives the
    // hooks, which receive it as their context.
    unsafe {
        let context = db.cast::<c_void>();
        ffi::sqlite3_preupdate_hook(db, Some(preupdate), context);
        ffi::sqlite3_commit_hook(db, Some(commit), context);
        ffi::sqlite3_rollback_hook(db, Some(rollback), context);
        ffi::sqlite3_trace_v2(db, ffi::SQLITE_TRACE_PROFILE, Some(profile), context);
    }
    ffi::SQLITE_OK
}

unsafe extern "C" fn preupdate(
    _context: *mut c_void,
    db: *mut ffi::sqlite3,
    op: c_int,
    _database: *const c_char,
    table: *const c_char,
    _old_rowid: ffi::sqlite3_int64,
    _new_rowid: ffi::sqlite3_int64,
) {
    let Some(op) = operation(op) else { return };
    if table.is_null() {
        return;
    }
    // SAFETY: SQLite owns the name for the duration of the callback.
    let Ok(name) = unsafe { CStr::from_ptr(table) }.to_str() else {
        return;
    };
    let declared = match SCHEMA.get().and_then(|schema| schema.get(name)) {
        Some(declared) => declared,
        None => return,
    };

    // SAFETY: the row is the one SQLite is reporting, and the declared indexes
    // were read from this table's schema.
    let (old, new) = unsafe {
        (
            if op.has_old() {
                read(db, &declared.indexes, true)
            } else {
                Vec::new()
            },
            if op.has_new() {
                read(db, &declared.indexes, false)
            } else {
                Vec::new()
            },
        )
    };
    let change = Change::record(declared.name, Arc::clone(&declared.columns), op, old, new);
    if let Ok(mut batches) = CHANGES.lock() {
        batches
            .entry(db as usize)
            .or_insert_with(Pending::new)
            .changes
            .push(change);
    }
}

unsafe extern "C" fn commit(context: *mut c_void) -> c_int {
    if let Ok(mut batches) = CHANGES.lock()
        && let Some(pending) = batches.get_mut(&(context as usize))
    {
        pending.committed = true;
    }
    0
}

unsafe extern "C" fn rollback(context: *mut c_void) {
    if let Ok(mut batches) = CHANGES.lock()
        && let Some(pending) = batches.get_mut(&(context as usize))
    {
        pending.changes.clear();
        pending.committed = false;
    }
}

/// The statement, including the commit it ended with, has finished.
///
/// `PROFILE` carries the elapsed time in `_elapsed`; the profile is only used as
/// an end-of-statement signal, so the value is ignored. It is the one point where
/// the commit is both done and still inside the statement, which is why the batch
/// is handed over here rather than from the commit hook. Only the connection that
/// ran the commit publishes its own batch.
unsafe extern "C" fn profile(
    _mask: std::ffi::c_uint,
    context: *mut c_void,
    _statement: *mut c_void,
    _elapsed: *mut c_void,
) -> c_int {
    let mut batches = match CHANGES.lock() {
        Ok(batches) => batches,
        Err(_) => return 0,
    };
    let Some(pending) = batches.get_mut(&(context as usize)) else {
        return 0;
    };
    if !pending.committed {
        return 0;
    }
    pending.committed = false;
    let batch = std::mem::take(&mut pending.changes);
    if !batch.is_empty()
        && let Some(sink) = SINK.get()
    {
        let _ = sink.send(batch);
    }
    0
}

fn operation(op: c_int) -> Option<RowOp> {
    match op {
        ffi::SQLITE_INSERT => Some(RowOp::Insert),
        ffi::SQLITE_UPDATE => Some(RowOp::Update),
        ffi::SQLITE_DELETE => Some(RowOp::Delete),
        _ => None,
    }
}

unsafe fn read(db: *mut ffi::sqlite3, indexes: &[i32], old: bool) -> Vec<Value> {
    indexes
        .iter()
        .map(|index| unsafe { value(db, *index, old) })
        .collect()
}

unsafe fn value(db: *mut ffi::sqlite3, index: i32, old: bool) -> Value {
    let mut cell: *mut ffi::sqlite3_value = ptr::null_mut();
    // SAFETY: `index` is a column of the row being reported, and `cell` is only
    // read while SQLite still owns the value.
    unsafe {
        let read = if old {
            ffi::sqlite3_preupdate_old(db, index, &mut cell)
        } else {
            ffi::sqlite3_preupdate_new(db, index, &mut cell)
        };
        if read != ffi::SQLITE_OK || cell.is_null() {
            return Value::Null;
        }
        match ffi::sqlite3_value_type(cell) {
            ffi::SQLITE_INTEGER => Value::Integer(ffi::sqlite3_value_int64(cell)),
            ffi::SQLITE_FLOAT => Value::Real(ffi::sqlite3_value_double(cell)),
            ffi::SQLITE_TEXT => {
                let text = ffi::sqlite3_value_text(cell);
                if text.is_null() {
                    return Value::Null;
                }
                let len = ffi::sqlite3_value_bytes(cell).max(0) as usize;
                let bytes = std::slice::from_raw_parts(text, len);
                Value::Text(String::from_utf8_lossy(bytes).into_owned())
            }
            _ => Value::Null,
        }
    }
}

/// Read each declared table's column layout, once, after migrations ran.
///
/// The declaration names columns and the observer reads a row by index, so this
/// is where the two meet. It runs on the migration connection, before the pool
/// opens, so a callback never has to look anything up: either the schema is
/// ready or the write is one the observer was installed too late for, which no
/// subscriber can be waiting on.
pub(crate) fn resolve(
    conn: &mut SqliteConnection,
    read_set: &'static [Table],
) -> Result<(), CatalogError> {
    let mut schema = HashMap::with_capacity(read_set.len());
    for table in read_set {
        let columns: Vec<Column> = diesel::sql_query("SELECT cid, name FROM pragma_table_info(?)")
            .bind::<Text, _>(table.name)
            .load(conn)
            .map_err(db_err)?;
        let mut indexes = Vec::with_capacity(table.columns.len());
        for column in table.columns {
            let Some(declared) = columns.iter().find(|found| found.name == *column) else {
                return Err(CatalogError::QueryFailed {
                    cause: format!("observe: {} has no column {column}", table.name),
                });
            };
            indexes.push(declared.cid);
        }
        schema.insert(
            table.name,
            Declared {
                name: table.name,
                columns: Arc::from(table.columns),
                indexes,
            },
        );
    }
    let _ = SCHEMA.set(schema);
    Ok(())
}

/// One row of `pragma_table_info`: a column's position and its name.
#[derive(QueryableByName)]
struct Column {
    #[diesel(sql_type = Integer)]
    cid: i32,
    #[diesel(sql_type = Text)]
    name: String,
}
