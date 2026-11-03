//! Storage behind an interface (plan P0-5), implemented on SQLite.
//!
//! A write stores the new record state and its history entry in one
//! transaction. The revision must follow the stored one exactly, so a stale
//! writer is refused instead of overwriting.

mod sqlite;

pub use sqlite::SqliteStore;

use std::path::Path;

/// What a backup file holds, shown before it replaces the current data.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BackupInfo {
    pub schema_version: i64,
    pub records: i64,
    pub history_entries: i64,
}

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
    #[error("not a TimeTrack backup")]
    NotABackup,
}

impl StoreError {
    /// Machine-readable code; the catalog entry is `error.<code>`.
    pub fn code(&self) -> &'static str {
        match self {
            StoreError::Storage(_) | StoreError::UnsupportedSchema(_) => "storage",
            StoreError::RevisionConflict { .. } => "revision_conflict",
            StoreError::NotFound => "not_found",
            StoreError::NotABackup => "not_a_backup",
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
    fn put(&mut self, record: &Record, entry: &HistoryEntry) -> Result<(), StoreError> {
        self.put_many(&[(record.clone(), entry.clone())])
    }
    /// Saves several records with their history entries in one transaction:
    /// either all are stored or none (used for the pieces of a split entry).
    fn put_many(&mut self, changes: &[(Record, HistoryEntry)]) -> Result<(), StoreError>;
    /// Records of a kind whose payload `work_date` lies in `[from, to]`
    /// (ISO dates, either bound optional), oldest first, tombstones excluded.
    fn list_by_date(
        &self,
        kind: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<Record>, StoreError>;
    /// Records of a kind whose payload `group_id` equals `group`.
    fn list_by_group(&self, kind: &str, group: &str) -> Result<Vec<Record>, StoreError>;
    /// Small installation-level values (identity), outside the record model.
    fn meta_get(&self, key: &str) -> Result<Option<String>, StoreError>;
    fn meta_set(&mut self, key: &str, value: &str) -> Result<(), StoreError>;
    /// Writes a consistent copy of everything to one file.
    fn backup_to(&self, path: &Path) -> Result<BackupInfo, StoreError>;
    /// Checks that a file is a readable, supported backup.
    fn inspect_backup(path: &Path) -> Result<BackupInfo, StoreError>
    where
        Self: Sized;
    /// Replaces everything with the contents of a backup file.
    fn restore_from(&mut self, path: &Path) -> Result<BackupInfo, StoreError>;
    fn get(&self, id: &Id) -> Result<Option<Record>, StoreError>;
    /// Records of a kind, oldest first, tombstones excluded unless asked.
    fn list(&self, kind: &str, include_deleted: bool) -> Result<Vec<Record>, StoreError>;
    /// A record's history, in revision order.
    fn history(&self, id: &Id) -> Result<Vec<HistoryEntry>, StoreError>;
}
