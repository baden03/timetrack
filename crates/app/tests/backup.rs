use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::json;
use timetrack_app::{App, ProjectInput};
use timetrack_storage::SqliteStore;

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("tt-app-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn open(path: &std::path::Path, now: &str) -> App<SqliteStore> {
    let t = DateTime::parse_from_rfc3339(now)
        .unwrap()
        .with_timezone(&Utc);
    App::open(
        SqliteStore::open(path).unwrap(),
        "Europe/Berlin",
        Arc::new(move || t),
    )
    .unwrap()
}

fn add_project(app: &mut App<SqliteStore>, name: &str) {
    let client = app.create_client(&format!("{name} client")).unwrap();
    app.create_project(ProjectInput {
        client_id: client.id,
        name: name.into(),
        description: String::new(),
        billable_default: true,
        category_ids: vec![],
    })
    .unwrap();
}

fn project_names(app: &App<SqliteStore>) -> Vec<String> {
    app.list("project", true)
        .unwrap()
        .iter()
        .map(|p| p.payload["name"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn backup_and_restore_through_the_service() {
    let d = dir("restore");
    let mut app = open(&d.join("db.sqlite"), "2026-11-03T09:00:00Z");
    assert_eq!(app.state().unwrap().last_backup_at, None);
    add_project(&mut app, "Alpha");
    let identity = app.identity().origin;

    let file = d.join("backup.sqlite");
    let info = app.backup(&file).unwrap();
    assert!(info.records >= 3);
    assert_eq!(
        app.state().unwrap().last_backup_at.as_deref(),
        Some("2026-11-03T09:00:00Z")
    );

    // More work after the backup, then a restore that goes back to it.
    add_project(&mut app, "Beta");
    assert_eq!(project_names(&app), ["Alpha", "Beta"]);
    let safety = d.join("safety.sqlite");
    app.restore(&file, &safety).unwrap();
    assert_eq!(project_names(&app), ["Alpha"]);
    assert_eq!(app.identity().origin, identity);
    // "Last backup" is the time of the file that was restored.
    assert_eq!(
        app.state().unwrap().last_backup_at.as_deref(),
        Some("2026-11-03T09:00:00Z")
    );

    // The safety copy holds what was there before the restore.
    let before = open(&safety, "2026-11-03T09:00:00Z");
    assert_eq!(project_names(&before), ["Alpha", "Beta"]);

    // The restored installation keeps working.
    add_project(&mut app, "Gamma");
    assert_eq!(project_names(&app), ["Alpha", "Gamma"]);
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn a_bad_file_is_refused_and_nothing_changes() {
    let d = dir("bad");
    let mut app = open(&d.join("db.sqlite"), "2026-11-03T09:00:00Z");
    add_project(&mut app, "Alpha");
    let junk = d.join("junk.txt");
    std::fs::write(&junk, "not a database").unwrap();
    let safety = d.join("safety.sqlite");
    let err = app.restore(&junk, &safety).unwrap_err();
    assert_eq!(err.code(), "not_a_backup");
    assert!(
        !safety.exists(),
        "no safety copy is made for a refused file"
    );
    assert_eq!(project_names(&app), ["Alpha"]);
    assert_eq!(
        app.inspect_backup(&junk).unwrap_err().code(),
        "not_a_backup"
    );
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn dispatch_backup_inspect_restore() {
    let d = dir("dispatch");
    let mut app = open(&d.join("db.sqlite"), "2026-11-03T09:00:00Z");
    add_project(&mut app, "Alpha");
    let file = d.join("b.sqlite");
    let info = app.dispatch("backup", &json!({"path": file})).unwrap();
    assert!(info["records"].as_i64().unwrap() >= 3);
    let looked = app
        .dispatch("inspect_backup", &json!({"path": file}))
        .unwrap();
    assert_eq!(looked["schema_version"], info["schema_version"]);
    add_project(&mut app, "Beta");
    app.dispatch("restore", &json!({"path": file})).unwrap();
    assert_eq!(project_names(&app), ["Alpha"]);
    // Without a named place, the safety copy sits next to the backup.
    assert!(d.join("b.sqlite.before-restore.sqlite").is_file());
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn settings_saved_before_the_reminder_existed_still_load() {
    let d = dir("settings");
    let mut app = open(&d.join("db.sqlite"), "2026-11-03T09:00:00Z");
    let rec = app.state().unwrap().settings;
    let mut old = rec.payload.clone();
    old.as_object_mut().unwrap().remove("backup_reminder_days");
    let parsed: timetrack_core::model::Settings = serde_json::from_value(old).unwrap();
    assert_eq!(parsed.backup_reminder_days, 7);
    let _ = &mut app;
    std::fs::remove_dir_all(&d).ok();
}
