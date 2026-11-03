//! Storage behind an interface (plan P0-5), implemented on SQLite.
//!
//! A write stores the new record state and its history entry in one
//! transaction. The revision must follow the stored one exactly, so a stale
//! writer is refused instead of overwriting.

mod sqlite;

pub use sqlite::SqliteStore;

use timetrack_core::id::Id;
use timetrack_core::record::{HistoryEntry, Record};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("revision conflict: stored {stored}, got {got}")]
    RevisionConflict { stored: u64, got: u64 },
    #[error("record not found")]
    NotFound,
    #[error("unsupported schema version {0}")]
    UnsupportedSchema(i64),
}

impl StoreError {
    /// Machine-readable code; the catalog entry is `error.<code>`.
    pub fn code(&self) -> &'static str {
        match self {
            StoreError::Storage(_) | StoreError::UnsupportedSchema(_) => "storage",
            StoreError::RevisionConflict { .. } => "revision_conflict",
            StoreError::NotFound => "not_found",
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Storage(e.to_string())
    }
}

pub trait Store {
    /// Saves a record state together with its history entry, atomically.
    fn put(&mut self, record: &Record, entry: &HistoryEntry) -> Result<(), StoreError>;
    fn get(&self, id: &Id) -> Result<Option<Record>, StoreError>;
    /// Records of a kind, oldest first, tombstones excluded unless asked.
    fn list(&self, kind: &str, include_deleted: bool) -> Result<Vec<Record>, StoreError>;
    /// A record's history, in revision order.
    fn history(&self, id: &Id) -> Result<Vec<HistoryEntry>, StoreError>;
}
