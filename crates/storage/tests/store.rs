use chrono::{DateTime, Utc};
use serde_json::json;
use timetrack_core::id::{IdGenerator, Origin};
use timetrack_core::record::{reconstruct, Actor, Record};
use timetrack_storage::{SqliteStore, Store, StoreError};

fn actor() -> Actor {
    Actor {
        author: "baden".into(),
        device: "mac-1".into(),
    }
}

fn at(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

/// The behaviour every Store implementation must show.
fn lifecycle(store: &mut dyn Store) {
    let ids = IdGenerator::new(Origin::new("test").unwrap());
    let (r1, h1) = Record::create(
        &ids,
        "client",
        json!({"name": "Acme"}),
        actor(),
        at("2026-11-03T09:00:00Z"),
    );
    let (r2, h2) = r1
        .revise(
            json!({"name": "Acme GmbH"}),
            actor(),
            at("2026-11-03T10:00:00Z"),
        )
        .unwrap();
    let (r3, h3) = r2.delete(actor(), at("2026-11-03T11:00:00Z")).unwrap();

    store.put(&r1, &h1).unwrap();
    assert_eq!(store.get(&r1.id).unwrap().unwrap(), r1);
    store.put(&r2, &h2).unwrap();
    assert_eq!(store.get(&r1.id).unwrap().unwrap(), r2);
    store.put(&r3, &h3).unwrap();

    let stored = store.get(&r1.id).unwrap().unwrap();
    assert!(stored.deleted);
    assert!(store.list("client", false).unwrap().is_empty());
    assert_eq!(store.list("client", true).unwrap().len(), 1);

    let history = store.history(&r1.id).unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(reconstruct(&history, None).unwrap(), stored);
    assert_eq!(reconstruct(&history, Some(2)).unwrap(), r2);

    // A stale writer is refused rather than overwriting.
    let (stale, stale_entry) = r1
        .revise(
            json!({"name": "Stale"}),
            actor(),
            at("2026-11-03T12:00:00Z"),
        )
        .unwrap();
    match store.put(&stale, &stale_entry) {
        Err(StoreError::RevisionConflict { stored: 3, got: 2 }) => {}
        other => panic!("expected revision conflict, got {other:?}"),
    }
}

#[test]
fn sqlite_lifecycle_in_memory() {
    lifecycle(&mut SqliteStore::open_in_memory().unwrap());
}

#[test]
fn sqlite_lifecycle_on_disk_survives_reopen() {
    let dir = std::env::temp_dir().join(format!("timetrack-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("db.sqlite");
    let ids = IdGenerator::new(Origin::new("disk").unwrap());
    let (r, h) = Record::create(
        &ids,
        "project",
        json!({"name": "Site"}),
        actor(),
        at("2026-11-03T09:00:00Z"),
    );
    {
        let mut store = SqliteStore::open(&path).unwrap();
        store.put(&r, &h).unwrap();
    }
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.get(&r.id).unwrap().unwrap(), r);
    assert_eq!(store.schema_version().unwrap(), 2);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn history_entry_is_written_with_the_record() {
    let mut store = SqliteStore::open_in_memory().unwrap();
    let ids = IdGenerator::new(Origin::random());
    let (r, h) = Record::create(&ids, "tag", json!({}), actor(), at("2026-11-03T09:00:00Z"));
    store.put(&r, &h).unwrap();
    assert_eq!(store.history(&r.id).unwrap().len(), 1);
}

#[test]
fn two_independent_databases_never_collide() {
    let mut a = SqliteStore::open_in_memory().unwrap();
    let mut b = SqliteStore::open_in_memory().unwrap();
    let ga = IdGenerator::new(Origin::random());
    let gb = IdGenerator::new(Origin::random());
    for i in 0..500 {
        let (r, h) = Record::create(
            &ga,
            "entry",
            json!({"n": i}),
            actor(),
            at("2026-11-03T09:00:00Z"),
        );
        a.put(&r, &h).unwrap();
        let (r, h) = Record::create(
            &gb,
            "entry",
            json!({"n": i}),
            actor(),
            at("2026-11-03T09:00:00Z"),
        );
        b.put(&r, &h).unwrap();
    }
    let ids_a: std::collections::HashSet<_> = a
        .list("entry", true)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();
    let ids_b: std::collections::HashSet<_> = b
        .list("entry", true)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(ids_a.len(), 500);
    assert!(ids_a.is_disjoint(&ids_b));
}

#[test]
fn put_many_is_all_or_nothing() {
    let mut store = SqliteStore::open_in_memory().unwrap();
    let ids = IdGenerator::new(Origin::random());
    let (a, ha) = Record::create(
        &ids,
        "entry",
        json!({"work_date": "2026-11-02"}),
        actor(),
        at("2026-11-03T09:00:00Z"),
    );
    let (b, hb) = Record::create(
        &ids,
        "entry",
        json!({"work_date": "2026-11-03"}),
        actor(),
        at("2026-11-03T09:00:00Z"),
    );
    store.put(&a, &ha).unwrap();
    // b is fine, but a stale copy of a is not: nothing may be stored.
    let (stale, hs) = a
        .revise(
            json!({"work_date": "2026-11-09"}),
            actor(),
            at("2026-11-03T10:00:00Z"),
        )
        .unwrap();
    store.put(&stale, &hs).unwrap();
    assert!(store.put_many(&[(b.clone(), hb), (stale, hs)]).is_err());
    assert!(store.get(&b.id).unwrap().is_none());
}

#[test]
fn queries_by_date_and_group_and_meta() {
    let mut store = SqliteStore::open_in_memory().unwrap();
    let ids = IdGenerator::new(Origin::random());
    for (d, g) in [
        ("2026-11-01", "g1"),
        ("2026-11-02", "g1"),
        ("2026-11-05", "g2"),
    ] {
        let (r, h) = Record::create(
            &ids,
            "entry",
            json!({"work_date": d, "group_id": g}),
            actor(),
            at("2026-11-06T09:00:00Z"),
        );
        store.put(&r, &h).unwrap();
    }
    let dates = |from, to| -> Vec<String> {
        store
            .list_by_date("entry", from, to)
            .unwrap()
            .iter()
            .map(|r| r.payload["work_date"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(dates(None, None).len(), 3);
    assert_eq!(
        dates(Some("2026-11-02"), Some("2026-11-04")),
        ["2026-11-02"]
    );
    assert_eq!(
        dates(Some("2026-11-02"), None),
        ["2026-11-02", "2026-11-05"]
    );
    assert_eq!(store.list_by_group("entry", "g1").unwrap().len(), 2);

    assert_eq!(store.meta_get("origin").unwrap(), None);
    store.meta_set("origin", "abc").unwrap();
    store.meta_set("origin", "def").unwrap();
    assert_eq!(store.meta_get("origin").unwrap().as_deref(), Some("def"));
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tt-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn filled(store: &mut SqliteStore, n: usize) -> Vec<timetrack_core::id::Id> {
    let ids = IdGenerator::new(Origin::random());
    (0..n)
        .map(|i| {
            let (r, h) = Record::create(
                &ids,
                "entry",
                json!({"work_date": "2026-11-02", "n": i}),
                actor(),
                at("2026-11-03T09:00:00Z"),
            );
            store.put(&r, &h).unwrap();
            r.id
        })
        .collect()
}

#[test]
fn backup_then_restore_reproduces_records_and_history_exactly() {
    let dir = temp_dir("backup");
    let mut original = SqliteStore::open(&dir.join("a.sqlite")).unwrap();
    let ids = filled(&mut original, 25);
    // One record with several revisions, to compare history.
    let rec = original.get(&ids[0]).unwrap().unwrap();
    let (r2, h2) = rec
        .revise(
            json!({"work_date": "2026-11-02", "n": 99}),
            actor(),
            at("2026-11-04T09:00:00Z"),
        )
        .unwrap();
    original.put(&r2, &h2).unwrap();
    original.meta_set("origin", "keepme").unwrap();

    let file = dir.join("backup.sqlite");
    let info = original.backup_to(&file).unwrap();
    assert_eq!((info.records, info.history_entries), (25, 26));
    // A single self-contained file.
    assert!(!dir.join("backup.sqlite-wal").exists());
    assert_eq!(SqliteStore::inspect_backup(&file).unwrap(), info);

    // Restore into a fresh installation that has other data of its own.
    let mut fresh = SqliteStore::open(&dir.join("b.sqlite")).unwrap();
    filled(&mut fresh, 3);
    fresh.restore_from(&file).unwrap();

    let list = |s: &SqliteStore| s.list("entry", true).unwrap();
    assert_eq!(list(&fresh), list(&original));
    for id in &ids {
        assert_eq!(fresh.history(id).unwrap(), original.history(id).unwrap());
    }
    assert_eq!(fresh.meta_get("origin").unwrap().as_deref(), Some("keepme"));
    // And it keeps working afterwards.
    filled(&mut fresh, 1);
    assert_eq!(fresh.list("entry", true).unwrap().len(), 26);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn things_that_are_not_backups_are_refused_and_change_nothing() {
    let dir = temp_dir("notbackup");
    let mut store = SqliteStore::open(&dir.join("a.sqlite")).unwrap();
    let ids = filled(&mut store, 2);

    let text = dir.join("notes.txt");
    std::fs::write(&text, "hello").unwrap();
    let empty_db = dir.join("empty.sqlite");
    rusqlite::Connection::open(&empty_db)
        .unwrap()
        .execute_batch("CREATE TABLE x (a)")
        .unwrap();
    let future = dir.join("future.sqlite");
    {
        let source = SqliteStore::open(&dir.join("src.sqlite")).unwrap();
        source.backup_to(&future).unwrap();
        rusqlite::Connection::open(&future)
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
    }
    for bad in [&text, &empty_db, &future, &dir.join("missing.sqlite")] {
        assert!(store.restore_from(bad).is_err(), "{bad:?}");
    }
    assert_eq!(store.list("entry", true).unwrap().len(), ids.len());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn an_older_database_is_copied_aside_before_it_is_migrated() {
    let dir = temp_dir("migrate");
    let path = dir.join("old.sqlite");
    {
        // A version 1 database, as the first release wrote it.
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE records (id TEXT PRIMARY KEY, kind TEXT NOT NULL, revision INTEGER NOT NULL,
                deleted INTEGER NOT NULL, created_by TEXT NOT NULL, updated_by TEXT NOT NULL,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL, payload TEXT NOT NULL);
             CREATE TABLE history (seq INTEGER PRIMARY KEY AUTOINCREMENT, record_id TEXT NOT NULL,
                kind TEXT NOT NULL, revision INTEGER NOT NULL, op TEXT NOT NULL, actor TEXT NOT NULL,
                at TEXT NOT NULL, before TEXT, after TEXT, UNIQUE (record_id, revision));
             PRAGMA user_version = 1;",
        )
        .unwrap();
    }
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), 2);
    let aside = dir.join("old.sqlite.pre-v1.bak");
    assert!(aside.is_file());
    assert_eq!(
        SqliteStore::inspect_backup(&aside).unwrap().schema_version,
        1
    );

    // An old backup restores into a current database and is migrated.
    let mut current = SqliteStore::open_in_memory().unwrap();
    let info = current.restore_from(&aside).unwrap();
    assert_eq!(info.schema_version, 2);
    std::fs::remove_dir_all(&dir).ok();
}
