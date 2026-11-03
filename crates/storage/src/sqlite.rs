use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use timetrack_core::id::Id;
use timetrack_core::record::{Actor, HistoryEntry, Op, Record};

use crate::{BackupInfo, Store, StoreError};

/// Ordered migrations. The database's `user_version` is the number applied.
const MIGRATIONS: &[&str] = &[
    "CREATE TABLE records (
        id          TEXT PRIMARY KEY,
        kind        TEXT NOT NULL,
        revision    INTEGER NOT NULL,
        deleted     INTEGER NOT NULL,
        created_by  TEXT NOT NULL,
        updated_by  TEXT NOT NULL,
        created_at  TEXT NOT NULL,
        updated_at  TEXT NOT NULL,
        payload     TEXT NOT NULL
    );
    CREATE INDEX records_kind ON records (kind, id);
    CREATE TABLE history (
        seq         INTEGER PRIMARY KEY AUTOINCREMENT,
        record_id   TEXT NOT NULL,
        kind        TEXT NOT NULL,
        revision    INTEGER NOT NULL,
        op          TEXT NOT NULL,
        actor       TEXT NOT NULL,
        at          TEXT NOT NULL,
        before      TEXT,
        after       TEXT,
        UNIQUE (record_id, revision)
    );
    -- History is append-only.
    CREATE TRIGGER history_no_update BEFORE UPDATE ON history
        BEGIN SELECT RAISE(ABORT, 'history is append-only'); END;
    CREATE TRIGGER history_no_delete BEFORE DELETE ON history
        BEGIN SELECT RAISE(ABORT, 'history is append-only'); END;",
    // 2: installation meta and indexed views of payload fields.
    "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    ALTER TABLE records ADD COLUMN work_date TEXT
        GENERATED ALWAYS AS (json_extract(payload, '$.work_date')) VIRTUAL;
    ALTER TABLE records ADD COLUMN group_id TEXT
        GENERATED ALWAYS AS (json_extract(payload, '$.group_id')) VIRTUAL;
    CREATE INDEX records_date ON records (kind, work_date);
    CREATE INDEX records_group ON records (kind, group_id);",
];

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Opens (creating if needed) the database file. A file from an older
    /// version is copied to `<file>.pre-v<N>.bak` before it is migrated, and
    /// the copy is kept (concept §16.3).
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        let current: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if current > 0 && current < MIGRATIONS.len() as i64 {
            let mut name = path.as_os_str().to_owned();
            name.push(format!(".pre-v{current}.bak"));
            copy_database(&conn, Path::new(&name))?;
        }
        Self::init(conn)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut store = SqliteStore { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn schema_version(&self) -> Result<i64, StoreError> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))?)
    }

    fn migrate(&mut self) -> Result<(), StoreError> {
        let current = self.schema_version()?;
        let target = MIGRATIONS.len() as i64;
        if current > target {
            return Err(StoreError::UnsupportedSchema(current));
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
            let tx = self.conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", i as i64 + 1)?;
            tx.commit()?;
        }
        Ok(())
    }
}

/// Copies a database into a new single file using SQLite's backup API, which
/// is safe while the database is in use. Written to a temporary name and
/// renamed, so a failed copy never replaces a good file.
fn copy_database(source: &Connection, path: &Path) -> Result<(), StoreError> {
    let mut tmp_name = path.as_os_str().to_owned();
    tmp_name.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp_name);
    let _ = std::fs::remove_file(&tmp);
    {
        let mut dest = Connection::open(&tmp)?;
        {
            let backup = rusqlite::backup::Backup::new(source, &mut dest)?;
            backup.run_to_completion(256, std::time::Duration::from_millis(5), None)?;
        }
        // One self-contained file, not a main file plus a write-ahead log.
        dest.pragma_update(None, "journal_mode", "DELETE")?;
    }
    std::fs::rename(&tmp, path).map_err(|e| StoreError::Storage(e.to_string()))?;
    Ok(())
}

fn count(conn: &Connection, table: &str) -> Result<i64, StoreError> {
    Ok(conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?)
}

fn info_of(conn: &Connection) -> Result<BackupInfo, StoreError> {
    Ok(BackupInfo {
        schema_version: conn.pragma_query_value(None, "user_version", |r| r.get(0))?,
        records: count(conn, "records")?,
        history_entries: count(conn, "history")?,
    })
}

fn json<T: serde::Serialize>(value: &T) -> Result<String, StoreError> {
    serde_json::to_string(value).map_err(|e| StoreError::Storage(e.to_string()))
}

fn from_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, StoreError> {
    serde_json::from_str(text).map_err(|e| StoreError::Storage(e.to_string()))
}

fn time(text: &str) -> Result<chrono::DateTime<chrono::Utc>, StoreError> {
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|t| t.with_timezone(&chrono::Utc))
        .map_err(|e| StoreError::Storage(e.to_string()))
}

fn stamp(t: chrono::DateTime<chrono::Utc>) -> String {
    t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn record_from_row(row: &rusqlite::Row<'_>) -> Result<Record, StoreError> {
    let id: String = row.get("id")?;
    let created_by: String = row.get("created_by")?;
    let updated_by: String = row.get("updated_by")?;
    let created_at: String = row.get("created_at")?;
    let updated_at: String = row.get("updated_at")?;
    let payload: String = row.get("payload")?;
    Ok(Record {
        id: Id::parse(&id).map_err(|e| StoreError::Storage(e.to_string()))?,
        kind: row.get("kind")?,
        revision: row.get::<_, i64>("revision")? as u64,
        deleted: row.get::<_, i64>("deleted")? != 0,
        created_by: from_json::<Actor>(&created_by)?,
        updated_by: from_json::<Actor>(&updated_by)?,
        created_at: time(&created_at)?,
        updated_at: time(&updated_at)?,
        payload: from_json(&payload)?,
    })
}

fn stored_revision(tx: &Transaction<'_>, id: &Id) -> Result<Option<u64>, StoreError> {
    Ok(tx
        .query_row(
            "SELECT revision FROM records WHERE id = ?1",
            [id.as_str()],
            |r| r.get::<_, i64>(0),
        )
        .optional()?
        .map(|v| v as u64))
}

fn put_in(tx: &Transaction<'_>, record: &Record, entry: &HistoryEntry) -> Result<(), StoreError> {
    if entry.record_id != record.id || entry.revision != record.revision {
        return Err(StoreError::Storage(
            "history entry does not match record".into(),
        ));
    }
    let stored = stored_revision(tx, &record.id)?;
    let expected = stored.map_or(1, |r| r + 1);
    if record.revision != expected {
        return Err(StoreError::RevisionConflict {
            stored: stored.unwrap_or(0),
            got: record.revision,
        });
    }
    tx.execute(
        "INSERT INTO records (id, kind, revision, deleted, created_by, updated_by,
                              created_at, updated_at, payload)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            revision = excluded.revision, deleted = excluded.deleted,
            updated_by = excluded.updated_by, updated_at = excluded.updated_at,
            payload = excluded.payload",
        params![
            record.id.as_str(),
            record.kind,
            record.revision as i64,
            record.deleted as i64,
            json(&record.created_by)?,
            json(&record.updated_by)?,
            stamp(record.created_at),
            stamp(record.updated_at),
            json(&record.payload)?,
        ],
    )?;
    tx.execute(
        "INSERT INTO history (record_id, kind, revision, op, actor, at, before, after)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            entry.record_id.as_str(),
            entry.kind,
            entry.revision as i64,
            entry.op.as_str(),
            json(&entry.actor)?,
            stamp(entry.at),
            entry.before.as_ref().map(json).transpose()?,
            entry.after.as_ref().map(json).transpose()?,
        ],
    )?;
    Ok(())
}

impl Store for SqliteStore {
    fn put_many(&mut self, changes: &[(Record, HistoryEntry)]) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        for (record, entry) in changes {
            put_in(&tx, record, entry)?;
        }
        tx.commit()?;
        Ok(())
    }

    fn list_by_date(
        &self,
        kind: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<Record>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM records
             WHERE kind = ?1 AND deleted = 0
               AND (?2 IS NULL OR work_date >= ?2) AND (?3 IS NULL OR work_date <= ?3)
             ORDER BY work_date, id",
        )?;
        let mut rows = stmt.query(params![kind, from, to])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(record_from_row(row)?);
        }
        Ok(out)
    }

    fn list_by_group(&self, kind: &str, group: &str) -> Result<Vec<Record>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM records WHERE kind = ?1 AND group_id = ?2 AND deleted = 0 ORDER BY id",
        )?;
        let mut rows = stmt.query(params![kind, group])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(record_from_row(row)?);
        }
        Ok(out)
    }

    fn meta_get(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()?)
    }

    fn meta_set(&mut self, key: &str, value: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn get(&self, id: &Id) -> Result<Option<Record>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT * FROM records WHERE id = ?1")?;
        let mut rows = stmt.query([id.as_str()])?;
        match rows.next()? {
            Some(row) => Ok(Some(record_from_row(row)?)),
            None => Ok(None),
        }
    }

    fn list(&self, kind: &str, include_deleted: bool) -> Result<Vec<Record>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM records WHERE kind = ?1 AND (?2 OR deleted = 0) ORDER BY id")?;
        let mut rows = stmt.query(params![kind, include_deleted])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(record_from_row(row)?);
        }
        Ok(out)
    }

    fn history(&self, id: &Id) -> Result<Vec<HistoryEntry>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, revision, op, actor, at, before, after
             FROM history WHERE record_id = ?1 ORDER BY revision",
        )?;
        let mut rows = stmt.query([id.as_str()])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let op: String = row.get(2)?;
            let actor: String = row.get(3)?;
            let at: String = row.get(4)?;
            let before: Option<String> = row.get(5)?;
            let after: Option<String> = row.get(6)?;
            out.push(HistoryEntry {
                record_id: id.clone(),
                kind: row.get(0)?,
                revision: row.get::<_, i64>(1)? as u64,
                op: Op::parse(&op).ok_or_else(|| StoreError::Storage(format!("bad op {op}")))?,
                actor: from_json(&actor)?,
                at: time(&at)?,
                before: before.as_deref().map(from_json).transpose()?,
                after: after.as_deref().map(from_json).transpose()?,
            });
        }
        Ok(out)
    }

    fn backup_to(&self, path: &Path) -> Result<BackupInfo, StoreError> {
        copy_database(&self.conn, path)?;
        info_of(&self.conn)
    }

    fn inspect_backup(path: &Path) -> Result<BackupInfo, StoreError> {
        use rusqlite::OpenFlags;
        if !path.is_file() {
            return Err(StoreError::NotABackup);
        }
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| StoreError::NotABackup)?;
        // Not a database at all fails here.
        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('records', 'history')",
                [],
                |r| r.get(0),
            )
            .map_err(|_| StoreError::NotABackup)?;
        if tables != 2 {
            return Err(StoreError::NotABackup);
        }
        let check: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        if check != "ok" {
            return Err(StoreError::NotABackup);
        }
        let info = info_of(&conn)?;
        if info.schema_version < 1 || info.schema_version > MIGRATIONS.len() as i64 {
            return Err(StoreError::UnsupportedSchema(info.schema_version));
        }
        Ok(info)
    }

    fn restore_from(&mut self, path: &Path) -> Result<BackupInfo, StoreError> {
        Self::inspect_backup(path)?;
        self.conn.restore(
            rusqlite::MAIN_DB,
            path,
            None::<fn(rusqlite::backup::Progress)>,
        )?;
        self.conn.pragma_update(None, "journal_mode", "WAL")?;
        // A backup from an older version is brought up to date.
        self.migrate()?;
        info_of(&self.conn)
    }
}
