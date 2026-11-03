use std::sync::Arc;

use chrono::{DateTime, Utc};
use timetrack_app::{App, AppError, EntryPatch, NewEntry, ProjectInput, Timing};
use timetrack_core::id::Id;
use timetrack_core::model::{Status, TimeEntry, KIND_CLIENT, KIND_ENTRY};
use timetrack_storage::SqliteStore;

struct Fixture {
    app: App<SqliteStore>,
    project: Id,
    category: Id,
}

fn clock_at(s: &str) -> timetrack_app::Clock {
    let t = DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc);
    Arc::new(move || t)
}

fn fixture() -> Fixture {
    let store = SqliteStore::open_in_memory().unwrap();
    // "Now" is after every date used below.
    let mut app = App::open(store, "Europe/Berlin", clock_at("2026-12-01T12:00:00Z")).unwrap();
    let client = app.create_client("Acme").unwrap();
    let category = app.create_category("Development").unwrap();
    let project = app
        .create_project(ProjectInput {
            client_id: client.id,
            name: "Website".into(),
            description: String::new(),
            billable_default: true,
            category_ids: vec![category.id.clone()],
        })
        .unwrap();
    Fixture {
        app,
        project: project.id,
        category: category.id,
    }
}

fn interval(f: &Fixture, date: &str, start: &str, end: &str) -> NewEntry {
    NewEntry {
        project_id: f.project.clone(),
        category_id: f.category.clone(),
        description: "Work".into(),
        billable: None,
        tags: vec![],
        timing: Timing::Interval {
            date: date.into(),
            start: start.into(),
            end: end.into(),
        },
        timezone: None,
    }
}

fn entry(rec: &timetrack_core::record::Record) -> TimeEntry {
    serde_json::from_value(rec.payload.clone()).unwrap()
}

#[test]
fn first_run_creates_identity_and_settings_once() {
    let dir = std::env::temp_dir().join(format!("tt-app-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("db.sqlite");
    let first = {
        let app = App::open(
            SqliteStore::open(&path).unwrap(),
            "Europe/Berlin",
            clock_at("2026-11-03T09:00:00Z"),
        )
        .unwrap();
        let state = app.state().unwrap();
        assert!(state.first_run);
        assert_eq!(state.settings.payload["timezone"], "Europe/Berlin");
        assert_eq!(state.settings.payload["entry_window_days"], 60);
        state.identity.origin
    };
    let mut app = App::open(
        SqliteStore::open(&path).unwrap(),
        "America/Costa_Rica",
        clock_at("2026-11-03T09:00:00Z"),
    )
    .unwrap();
    let state = app.state().unwrap();
    assert_eq!(state.identity.origin, first);
    assert_eq!(state.settings.payload["timezone"], "Europe/Berlin"); // not replaced
    app.finish_first_run().unwrap();
    assert!(!app.state().unwrap().first_run);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn unknown_system_zone_falls_back_to_utc() {
    let app = App::open(
        SqliteStore::open_in_memory().unwrap(),
        "Not/AZone",
        clock_at("2026-11-03T09:00:00Z"),
    )
    .unwrap();
    assert_eq!(app.state().unwrap().settings.payload["timezone"], "UTC");
}

#[test]
fn reference_data_archive_restore_and_guarded_delete() {
    let mut f = fixture();
    let client = f.app.list(KIND_CLIENT, false).unwrap().remove(0);
    // A project exists, so the client cannot be deleted.
    assert!(matches!(
        f.app.delete_client(&client.id),
        Err(AppError::StillReferenced)
    ));
    f.app
        .set_client_status(&client.id, Status::Archived)
        .unwrap();
    assert!(f.app.list(KIND_CLIENT, false).unwrap().is_empty());
    assert_eq!(f.app.list(KIND_CLIENT, true).unwrap().len(), 1);
    f.app.set_client_status(&client.id, Status::Active).unwrap();

    f.app
        .create_entry(interval(&f, "2026-11-02", "09:00", "10:00"))
        .unwrap();
    assert!(matches!(
        f.app.delete_project(&f.project),
        Err(AppError::StillReferenced)
    ));
    assert!(matches!(
        f.app.delete_category(&f.category),
        Err(AppError::StillReferenced)
    ));

    let spare = f.app.create_category("Spare").unwrap();
    f.app.delete_category(&spare.id).unwrap();
    assert!(matches!(
        f.app.delete_category(&spare.id),
        Err(AppError::NotFound)
    ));
}

#[test]
fn names_are_required() {
    let mut f = fixture();
    assert_eq!(
        f.app.create_client("  ").unwrap_err().code(),
        "name_required"
    );
}

#[test]
fn simple_entry_is_stored_with_zone_and_offsets() {
    let mut f = fixture();
    let saved = f
        .app
        .create_entry(interval(&f, "2026-11-02", "09:15", "12:45"))
        .unwrap();
    assert_eq!(saved.pieces.len(), 1);
    let e = entry(&saved.pieces[0]);
    assert_eq!(e.minutes, 210);
    assert_eq!(e.timezone, "Europe/Berlin");
    assert_eq!(e.start_offset, Some(60));
    assert!(e.billable); // from the project
    assert_eq!(e.work_date.to_string(), "2026-11-02");
}

#[test]
fn midnight_split_and_dst_vector_through_the_service() {
    let mut f = fixture();
    let saved = f
        .app
        .create_entry(interval(&f, "2026-10-24", "23:00", "04:00"))
        .unwrap();
    let pieces: Vec<TimeEntry> = saved.pieces.iter().map(entry).collect();
    assert_eq!(pieces.len(), 2);
    assert_eq!(
        (pieces[0].work_date.to_string(), pieces[0].minutes),
        ("2026-10-24".into(), 60)
    );
    assert_eq!(
        (pieces[1].work_date.to_string(), pieces[1].minutes),
        ("2026-10-25".into(), 300)
    );
    assert_eq!(pieces[0].group_id, pieces[1].group_id);
    assert_eq!(pieces[0].start_offset, Some(120));
    assert_eq!(pieces[1].end_offset, Some(60));
    assert_eq!(pieces[1].piece_count, 2);
}

#[test]
fn costa_rica_author_keeps_their_date() {
    let mut f = fixture();
    let mut new = interval(&f, "2026-11-02", "22:00", "23:30");
    new.timezone = Some("America/Costa_Rica".into());
    let saved = f.app.create_entry(new).unwrap();
    let e = entry(&saved.pieces[0]);
    assert_eq!(
        (e.work_date.to_string(), e.minutes),
        ("2026-11-02".into(), 90)
    );
}

#[test]
fn editing_timing_resplits_and_merges_the_group() {
    let mut f = fixture();
    let saved = f
        .app
        .create_entry(interval(&f, "2026-11-02", "23:00", "02:00"))
        .unwrap();
    let group = entry(&saved.pieces[0]).group_id;
    assert_eq!(saved.pieces.len(), 2);

    // End moves to 23:30: the pieces merge into one, the second is tombstoned.
    let merged = f
        .app
        .update_entry(
            &group,
            EntryPatch {
                timing: Some(Timing::Interval {
                    date: "2026-11-02".into(),
                    start: "23:00".into(),
                    end: "23:30".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(merged.pieces.len(), 1);
    assert_eq!(entry(&merged.pieces[0]).minutes, 30);
    assert_eq!(f.app.list_entries(None, None).unwrap().len(), 1);

    // And back to a split entry again: a new piece record appears.
    let again = f
        .app
        .update_entry(
            &group,
            EntryPatch {
                timing: Some(Timing::Interval {
                    date: "2026-11-02".into(),
                    start: "23:00".into(),
                    end: "01:00".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(again.pieces.len(), 2);
    assert_eq!(f.app.list_entries(None, None).unwrap().len(), 2);
}

#[test]
fn text_edit_does_not_touch_timing_and_history_shows_before_and_after() {
    let mut f = fixture();
    let saved = f
        .app
        .create_entry(interval(&f, "2026-11-02", "23:00", "02:00"))
        .unwrap();
    let group = entry(&saved.pieces[0]).group_id;
    let before: Vec<TimeEntry> = saved.pieces.iter().map(entry).collect();

    let edited = f
        .app
        .update_entry(
            &group,
            EntryPatch {
                description: Some("New text".into()),
                tags: Some(vec!["a".into(), "A".into(), "b".into()]),
                ..Default::default()
            },
        )
        .unwrap();
    for (old, new) in before.iter().zip(edited.pieces.iter().map(entry)) {
        assert_eq!(new.description, "New text");
        assert_eq!(new.tags, ["a", "b"]);
        assert_eq!(
            (old.start_at, old.end_at, old.minutes),
            (new.start_at, new.end_at, new.minutes)
        );
    }
    let history = f.app.history(&saved.pieces[0].id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].before.as_ref().unwrap()["description"], "Work");
    assert_eq!(
        history[1].after.as_ref().unwrap()["description"],
        "New text"
    );
    assert_eq!(edited.pieces[0].revision, 2);
}

#[test]
fn validation_rules() {
    let mut f = fixture();
    let code = |f: &mut Fixture, n: NewEntry| f.app.create_entry(n).unwrap_err().code();
    assert_eq!(
        {
            let n = interval(&f, "2026-11-02", "10:00", "10:00");
            code(&mut f, n)
        },
        "end_not_after_start"
    );
    assert_eq!(
        {
            let n = interval(&f, "2026-12-01", "11:00", "14:00");
            code(&mut f, n)
        },
        "entry_in_future"
    );
    assert_eq!(
        {
            let n = interval(&f, "2026-11-02", "9am", "10:00");
            code(&mut f, n)
        },
        "invalid_date_time"
    );
    assert_eq!(
        {
            let n = interval(&f, "2026-03-29", "02:30", "04:00");
            code(&mut f, n)
        },
        "nonexistent_local_time"
    );

    let mut blank = interval(&f, "2026-11-02", "09:00", "10:00");
    blank.description = "   ".into();
    assert_eq!(code(&mut f, blank), "description_required");

    let mut many = interval(&f, "2026-11-02", "09:00", "10:00");
    many.tags = (0..11).map(|i| format!("t{i}")).collect();
    assert_eq!(code(&mut f, many), "too_many_tags");

    let other = f.app.create_category("Design").unwrap();
    let mut wrong = interval(&f, "2026-11-02", "09:00", "10:00");
    wrong.category_id = other.id;
    assert_eq!(code(&mut f, wrong), "category_not_enabled");

    f.app
        .set_project_status(&f.project, Status::Archived)
        .unwrap();
    assert_eq!(
        {
            let n = interval(&f, "2026-11-02", "09:00", "10:00");
            code(&mut f, n)
        },
        "project_archived"
    );
    assert!(f.app.list_entries(None, None).unwrap().is_empty());
}

#[test]
fn duration_entries() {
    let mut f = fixture();
    let mut new = interval(&f, "2026-11-02", "09:00", "10:00");
    new.timing = Timing::Duration {
        date: "2026-11-02".into(),
        minutes: 90,
    };
    let saved = f.app.create_entry(new.clone()).unwrap();
    let e = entry(&saved.pieces[0]);
    assert_eq!((e.minutes, e.start_at), (90, None));

    new.timing = Timing::Duration {
        date: "2026-11-02".into(),
        minutes: 1441,
    };
    assert_eq!(
        f.app.create_entry(new.clone()).unwrap_err().code(),
        "duration_out_of_range"
    );
    new.timing = Timing::Duration {
        date: "2026-12-02".into(),
        minutes: 30,
    };
    assert_eq!(
        f.app.create_entry(new).unwrap_err().code(),
        "entry_in_future"
    );
}

#[test]
fn overlap_warns_but_saves_and_duration_never_overlaps() {
    let mut f = fixture();
    let first = f
        .app
        .create_entry(interval(&f, "2026-11-02", "09:00", "11:00"))
        .unwrap();
    let second = f
        .app
        .create_entry(interval(&f, "2026-11-02", "10:00", "12:00"))
        .unwrap();
    assert_eq!(second.overlaps, [entry(&first.pieces[0]).group_id]);
    let adjacent = f
        .app
        .create_entry(interval(&f, "2026-11-02", "12:00", "13:00"))
        .unwrap();
    assert!(adjacent.overlaps.is_empty());

    let mut dur = interval(&f, "2026-11-02", "09:00", "10:00");
    dur.timing = Timing::Duration {
        date: "2026-11-02".into(),
        minutes: 60,
    };
    assert!(f.app.create_entry(dur).unwrap().overlaps.is_empty());
}

#[test]
fn delete_entry_removes_every_piece() {
    let mut f = fixture();
    let saved = f
        .app
        .create_entry(interval(&f, "2026-11-02", "23:00", "02:00"))
        .unwrap();
    let group = entry(&saved.pieces[0]).group_id;
    f.app.delete_entry(&group).unwrap();
    assert!(f.app.list_entries(None, None).unwrap().is_empty());
    assert!(matches!(
        f.app.delete_entry(&group),
        Err(AppError::NotFound)
    ));
    // The history still holds the deleted pieces.
    assert_eq!(f.app.history(&saved.pieces[1].id).unwrap().len(), 2);
    assert_eq!(KIND_ENTRY, "entry");
}

#[test]
fn suggestions_follow_the_day_and_the_project() {
    let mut f = fixture();
    let none = f.app.suggest(&f.project, "2026-11-02").unwrap();
    assert_eq!((none.start, none.category_id), (None, None));
    f.app
        .create_entry(interval(&f, "2026-11-02", "09:00", "11:30"))
        .unwrap();
    let s = f.app.suggest(&f.project, "2026-11-02").unwrap();
    assert_eq!(s.start.as_deref(), Some("11:30"));
    assert_eq!(s.category_id, Some(f.category.clone()));
    assert_eq!(f.app.suggest(&f.project, "2026-11-03").unwrap().start, None);
}

#[test]
fn list_entries_filters_by_date_window() {
    let mut f = fixture();
    for d in ["2026-10-01", "2026-11-02", "2026-11-20"] {
        f.app
            .create_entry(interval(&f, d, "09:00", "10:00"))
            .unwrap();
    }
    assert_eq!(
        f.app
            .list_entries(Some("2026-11-01"), Some("2026-11-30"))
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn dispatch_round_trip_and_errors() {
    use serde_json::json;
    let mut f = fixture();
    let client = f
        .app
        .dispatch("create_client", &json!({"name": "Beta"}))
        .unwrap();
    assert_eq!(client["payload"]["name"], "Beta");
    let saved = f
        .app
        .dispatch(
            "create_entry",
            &json!({"entry": {
                "project_id": f.project, "category_id": f.category, "description": "x",
                "timing": {"mode": "duration", "date": "2026-11-02", "minutes": 45}
            }}),
        )
        .unwrap();
    assert_eq!(saved["pieces"][0]["payload"]["minutes"], 45);
    let listed = f
        .app
        .dispatch("list_entries", &json!({"from": "2026-11-01"}))
        .unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(
        f.app.dispatch("nope", &json!({})).unwrap_err().code(),
        "unknown_command"
    );
    assert_eq!(
        f.app
            .dispatch("create_client", &json!({}))
            .unwrap_err()
            .code(),
        "invalid_arguments"
    );
}

#[test]
fn monthly_summary_is_billable_only_and_splits_at_month_end() {
    let mut f = fixture();
    // 31 Oct 23:00 to 1 Nov 01:00 contributes 1:00 to October and 1:00 to November.
    f.app
        .create_entry(interval(&f, "2026-10-31", "23:00", "01:00"))
        .unwrap();
    f.app
        .create_entry(interval(&f, "2026-11-02", "09:00", "10:30"))
        .unwrap();
    let mut private = interval(&f, "2026-11-02", "11:00", "12:00");
    private.billable = Some(false);
    private.description = "Not billed".into();
    f.app.create_entry(private).unwrap();

    let nov = f.app.monthly_summary("2026-11").unwrap();
    assert_eq!(nov.summary.minutes, 60 + 90);
    assert!(nov.text.contains("TimeTrack summary — November 2026"));
    assert!(nov.text.contains("ACME"));
    assert!(!nov.text.contains("Not billed"));
    let oct = f.app.monthly_summary("2026-10").unwrap();
    assert_eq!(oct.summary.minutes, 60);
    assert!(f
        .app
        .monthly_summary("2026-12")
        .unwrap()
        .text
        .contains("No billable time"));
    assert_eq!(
        f.app.monthly_summary("nonsense").unwrap_err().code(),
        "invalid_month"
    );
}

#[test]
fn search_filters_and_totals() {
    use timetrack_app::SearchQuery;
    let mut f = fixture();
    let mut a = interval(&f, "2026-11-02", "09:00", "10:00");
    a.description = "Fix Safari bug".into();
    a.tags = vec!["Bug".into()];
    f.app.create_entry(a).unwrap();
    let mut b = interval(&f, "2026-11-03", "09:00", "11:00");
    b.description = "Kickoff meeting".into();
    b.billable = Some(false);
    f.app.create_entry(b).unwrap();

    let q = |f: &Fixture, q: SearchQuery| f.app.search_entries(&q).unwrap();
    let all = q(&f, SearchQuery::default());
    assert_eq!((all.pieces.len(), all.total_minutes), (2, 180));
    assert_eq!(all.pieces[0].payload["work_date"], "2026-11-03"); // newest first
    let text = q(
        &f,
        SearchQuery {
            text: Some("safari".into()),
            ..Default::default()
        },
    );
    assert_eq!(text.total_minutes, 60);
    let tag = q(
        &f,
        SearchQuery {
            tag: Some("bug".into()),
            ..Default::default()
        },
    );
    assert_eq!(tag.pieces.len(), 1);
    let unbilled = q(
        &f,
        SearchQuery {
            billable: Some(false),
            ..Default::default()
        },
    );
    assert_eq!(unbilled.total_minutes, 120);
    let dated = q(
        &f,
        SearchQuery {
            from: Some("2026-11-03".into()),
            ..Default::default()
        },
    );
    assert_eq!(dated.pieces.len(), 1);
    let none = q(
        &f,
        SearchQuery {
            text: Some("zzz".into()),
            ..Default::default()
        },
    );
    assert_eq!(none.pieces.len(), 0);

    let client = f.app.list(KIND_CLIENT, false).unwrap().remove(0);
    let by_client = q(
        &f,
        SearchQuery {
            client_id: Some(client.id),
            ..Default::default()
        },
    );
    assert_eq!(by_client.pieces.len(), 2);
}
