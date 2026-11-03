//! Time-zone and calendar edge cases driven through the real service
//! (R1 acceptance criterion 3 and the hardening list in the development plan).

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use timetrack_app::{App, EntryPatch, NewEntry, ProjectInput, StartTimer, StopTimer, Timing};
use timetrack_core::id::Id;
use timetrack_core::model::{Settings, TimeEntry};
use timetrack_storage::SqliteStore;

type Now = Arc<Mutex<DateTime<Utc>>>;

fn at(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

struct S {
    app: App<SqliteStore>,
    now: Now,
    project: Id,
}

fn setup(now: &str) -> S {
    let now: Now = Arc::new(Mutex::new(at(now)));
    let clock = {
        let now = now.clone();
        Arc::new(move || *now.lock().unwrap())
    };
    let mut app = App::open(
        SqliteStore::open_in_memory().unwrap(),
        "Europe/Berlin",
        clock,
    )
    .unwrap();
    let client = app.create_client("Acme").unwrap();
    let project = app
        .create_project(ProjectInput {
            client_id: client.id,
            name: "Site".into(),
            description: String::new(),
            billable_default: true,
            category_ids: vec![],
        })
        .unwrap();
    S {
        app,
        now,
        project: project.id,
    }
}

fn add(s: &mut S, date: &str, start: &str, end: &str, zone: Option<&str>) -> Vec<TimeEntry> {
    let saved = s
        .app
        .create_entry(NewEntry {
            project_id: s.project.clone(),
            category_id: None,
            description: "work".into(),
            billable: None,
            tags: vec![],
            timing: Timing::Interval {
                date: date.into(),
                start: start.into(),
                end: end.into(),
            },
            timezone: zone.map(Into::into),
        })
        .unwrap();
    saved
        .pieces
        .iter()
        .map(|p| serde_json::from_value(p.payload.clone()).unwrap())
        .collect()
}

fn minutes(pieces: &[TimeEntry]) -> Vec<i64> {
    pieces.iter().map(|p| p.minutes).collect()
}

#[test]
fn spring_forward_loses_the_skipped_hour() {
    let mut s = setup("2026-04-01T12:00:00Z");
    // 02:00 to 03:00 does not exist on 29 March in Berlin.
    assert_eq!(
        minutes(&add(&mut s, "2026-03-29", "01:30", "03:30", None)),
        [60]
    );
    let err = s
        .app
        .create_entry(NewEntry {
            project_id: s.project.clone(),
            category_id: None,
            description: "x".into(),
            billable: None,
            tags: vec![],
            timing: Timing::Interval {
                date: "2026-03-29".into(),
                start: "02:30".into(),
                end: "04:00".into(),
            },
            timezone: None,
        })
        .unwrap_err();
    assert_eq!(err.code(), "nonexistent_local_time");
}

#[test]
fn fall_back_repeats_an_hour_and_the_timer_counts_real_time() {
    let mut s = setup("2026-10-24T22:30:00Z"); // 00:30 CEST on 25 Oct
    s.app
        .start_timer(StartTimer {
            project_id: s.project.clone(),
            category_id: None,
            description: String::new(),
            tags: vec![],
            billable: None,
        })
        .unwrap();
    *s.now.lock().unwrap() = at("2026-10-25T03:00:00Z"); // 04:00 CET
    let saved = s
        .app
        .stop_timer(StopTimer {
            description: "Long night".into(),
            end_time: None,
            tags: None,
            category_id: None,
        })
        .unwrap();
    // Wall clock says 3:30; four and a half real hours passed.
    assert_eq!(saved.pieces[0].payload["minutes"], 270);
    assert_eq!(saved.pieces[0].payload["start_offset"], 120);
    assert_eq!(saved.pieces[0].payload["end_offset"], 60);
}

#[test]
fn month_and_year_ends_split_into_the_right_months() {
    let mut s = setup("2028-03-01T12:00:00Z");
    // End of October, end of December and a leap day.
    assert_eq!(
        minutes(&add(&mut s, "2026-10-31", "23:00", "01:00", None)),
        [60, 60]
    );
    assert_eq!(
        minutes(&add(&mut s, "2026-12-31", "23:30", "00:30", None)),
        [30, 30]
    );
    assert_eq!(
        minutes(&add(&mut s, "2028-02-28", "23:00", "01:00", None)),
        [60, 60]
    );

    let total = |s: &S, month: &str| s.app.monthly_summary(month).unwrap().summary.minutes;
    assert_eq!(total(&s, "2026-10"), 60);
    assert_eq!(total(&s, "2026-11"), 60);
    assert_eq!(total(&s, "2026-12"), 30);
    assert_eq!(total(&s, "2027-01"), 30);
    assert_eq!(total(&s, "2028-02"), 120); // 28 Feb 23:00 to 29 Feb 01:00 stays in February
    assert_eq!(total(&s, "2028-03"), 0);
    assert!(s
        .app
        .monthly_summary("2027-02")
        .unwrap()
        .text
        .contains("No billable time"));
}

#[test]
fn another_zone_keeps_its_own_dates_and_edits_use_the_entry_zone() {
    let mut s = setup("2026-11-10T12:00:00Z");
    // A Costa Rica author, evening of 2 Nov (UTC is already 3 Nov).
    let pieces = add(
        &mut s,
        "2026-11-02",
        "22:00",
        "23:30",
        Some("America/Costa_Rica"),
    );
    assert_eq!(pieces[0].work_date.to_string(), "2026-11-02");
    let group = pieces[0].group_id.clone();

    // The device moves to Berlin; the entry keeps its own zone and date.
    let mut settings: Settings =
        serde_json::from_value(s.app.state().unwrap().settings.payload).unwrap();
    settings.timezone = "Europe/Berlin".into();
    s.app.update_settings(settings).unwrap();
    let edited = s
        .app
        .update_entry(
            &group,
            EntryPatch {
                timing: Some(Timing::Interval {
                    date: "2026-11-02".into(),
                    start: "22:00".into(),
                    end: "23:45".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let e: TimeEntry = serde_json::from_value(edited.pieces[0].payload.clone()).unwrap();
    assert_eq!(
        (e.timezone.as_str(), e.minutes, e.work_date.to_string()),
        ("America/Costa_Rica", 105, "2026-11-02".into())
    );
    assert_eq!(e.start_offset, Some(-360));

    // Entries made now use the new zone.
    let fresh = add(&mut s, "2026-11-02", "09:00", "10:00", None);
    assert_eq!(fresh[0].timezone, "Europe/Berlin");
    assert_eq!(fresh[0].start_offset, Some(60));
}
