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
    assert_eq!(store.schema_version().unwrap(), 1);
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
