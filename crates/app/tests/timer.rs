use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use timetrack_app::{App, Clock, ProjectInput, StartTimer, StopTimer};
use timetrack_core::id::Id;
use timetrack_core::model::TimeEntry;
use timetrack_storage::SqliteStore;

type Now = Arc<Mutex<DateTime<Utc>>>;

fn at(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

fn clock(now: &Now) -> Clock {
    let now = now.clone();
    Arc::new(move || *now.lock().unwrap())
}

struct Setup {
    app: App<SqliteStore>,
    now: Now,
    project: Id,
    category: Id,
}

fn setup(start: &str) -> Setup {
    let now: Now = Arc::new(Mutex::new(at(start)));
    let mut app = App::open(
        SqliteStore::open_in_memory().unwrap(),
        "Europe/Berlin",
        clock(&now),
    )
    .unwrap();
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
    Setup {
        app,
        now,
        project: project.id,
        category: category.id,
    }
}

fn start(s: &mut Setup) -> timetrack_core::record::Record {
    s.app
        .start_timer(StartTimer {
            project_id: s.project.clone(),
            category_id: s.category.clone(),
            description: String::new(),
            tags: vec![],
            billable: None,
        })
        .unwrap()
}

fn stop(description: &str, end_time: Option<&str>) -> StopTimer {
    StopTimer {
        description: description.into(),
        end_time: end_time.map(Into::into),
        tags: None,
    }
}

fn advance(s: &Setup, to: &str) {
    *s.now.lock().unwrap() = at(to);
}

#[test]
fn start_is_to_the_minute_and_only_one_runs() {
    let mut s = setup("2026-11-03T09:00:42Z");
    let rec = start(&mut s);
    assert_eq!(rec.payload["start_at"], "2026-11-03T09:00:00Z");
    let state = s.app.timer_state().unwrap();
    assert!(state.timer.is_some() && !state.stale);
    let again = s.app.start_timer(StartTimer {
        project_id: s.project.clone(),
        category_id: s.category.clone(),
        description: String::new(),
        tags: vec![],
        billable: None,
    });
    assert_eq!(again.unwrap_err().code(), "timer_running");
}

#[test]
fn stop_creates_an_entry_and_removes_the_timer_atomically() {
    let mut s = setup("2026-11-03T09:00:00Z");
    start(&mut s);
    s.app
        .update_timer(Some("typed while running".into()), Some(vec!["a".into()]))
        .unwrap();
    advance(&s, "2026-11-03T10:30:20Z");

    // The description is required; a refused stop leaves the timer alone.
    assert_eq!(
        s.app.stop_timer(stop("  ", None)).unwrap_err().code(),
        "description_required"
    );
    assert!(s.app.timer_state().unwrap().timer.is_some());

    let saved = s.app.stop_timer(stop("Built the form", None)).unwrap();
    let e: TimeEntry = serde_json::from_value(saved.pieces[0].payload.clone()).unwrap();
    assert_eq!(
        (e.minutes, e.description.as_str(), e.tags.clone()),
        (90, "Built the form", vec!["a".to_string()])
    );
    assert!(s.app.timer_state().unwrap().timer.is_none());
    assert_eq!(s.app.list_entries(None, None).unwrap().len(), 1);
    assert_eq!(
        s.app.stop_timer(stop("x", None)).unwrap_err().code(),
        "no_timer"
    );
}

#[test]
fn timer_survives_a_restart() {
    let dir = std::env::temp_dir().join(format!("tt-timer-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("db.sqlite");
    let now: Now = Arc::new(Mutex::new(at("2026-11-03T09:00:00Z")));
    let (project, category) = {
        let mut app = App::open(
            SqliteStore::open(&path).unwrap(),
            "Europe/Berlin",
            clock(&now),
        )
        .unwrap();
        let client = app.create_client("Acme").unwrap();
        let category = app.create_category("Dev").unwrap();
        let project = app
            .create_project(ProjectInput {
                client_id: client.id,
                name: "Site".into(),
                description: String::new(),
                billable_default: true,
                category_ids: vec![category.id.clone()],
            })
            .unwrap();
        app.start_timer(StartTimer {
            project_id: project.id.clone(),
            category_id: category.id.clone(),
            description: "draft".into(),
            tags: vec![],
            billable: None,
        })
        .unwrap();
        (project.id, category.id)
    }; // the app is dropped without stopping: a quit or a crash
    let _ = (project, category);
    *now.lock().unwrap() = at("2026-11-03T11:00:00Z");
    let mut app = App::open(
        SqliteStore::open(&path).unwrap(),
        "Europe/Berlin",
        clock(&now),
    )
    .unwrap();
    let state = app.timer_state().unwrap();
    assert_eq!(
        state.timer.as_ref().unwrap().payload["description"],
        "draft"
    );
    assert!(!state.stale);
    let saved = app.stop_timer(stop("Recovered", None)).unwrap();
    assert_eq!(saved.pieces[0].payload["minutes"], 120);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn stale_timer_is_flagged_and_can_be_stopped_at_the_real_time_or_discarded() {
    let mut s = setup("2026-11-02T08:00:00Z"); // 09:00 Berlin on 2 Nov
    start(&mut s);
    advance(&s, "2026-11-03T08:00:00Z");
    assert!(s.app.timer_state().unwrap().stale);

    let saved = s
        .app
        .stop_timer(stop("Worked until half past five", Some("17:30")))
        .unwrap();
    assert_eq!(saved.pieces.len(), 1);
    assert_eq!(saved.pieces[0].payload["work_date"], "2026-11-02");
    assert_eq!(saved.pieces[0].payload["minutes"], 510);

    start(&mut s);
    s.app.discard_timer().unwrap();
    assert!(s.app.timer_state().unwrap().timer.is_none());
    assert_eq!(s.app.discard_timer().unwrap_err().code(), "no_timer");
    assert_eq!(s.app.list_entries(None, None).unwrap().len(), 1);
}

#[test]
fn a_timer_across_midnight_is_split_on_stop() {
    let mut s = setup("2026-11-02T22:00:00Z"); // 23:00 Berlin
    start(&mut s);
    advance(&s, "2026-11-03T01:00:00Z"); // 02:00 Berlin
    let saved = s.app.stop_timer(stop("Night shift", None)).unwrap();
    let minutes: Vec<i64> = saved
        .pieces
        .iter()
        .map(|p| p.payload["minutes"].as_i64().unwrap())
        .collect();
    assert_eq!(minutes, [60, 120]);
}

#[test]
fn stopping_in_the_same_minute_is_refused_and_keeps_the_timer() {
    let mut s = setup("2026-11-03T09:00:10Z");
    start(&mut s);
    advance(&s, "2026-11-03T09:00:50Z");
    assert_eq!(
        s.app.stop_timer(stop("x", None)).unwrap_err().code(),
        "end_not_after_start"
    );
    assert!(s.app.timer_state().unwrap().timer.is_some());
}

#[test]
fn quick_start_repeats_the_most_recent_project_and_category() {
    let mut s = setup("2026-11-03T12:00:00Z");
    assert_eq!(
        s.app.quick_start_timer().unwrap_err().code(),
        "nothing_to_repeat"
    );
    s.app
        .create_entry(timetrack_app::NewEntry {
            project_id: s.project.clone(),
            category_id: s.category.clone(),
            description: "earlier".into(),
            billable: Some(false),
            tags: vec![],
            timing: timetrack_app::Timing::Duration {
                date: "2026-11-03".into(),
                minutes: 30,
            },
            timezone: None,
        })
        .unwrap();
    let rec = s.app.quick_start_timer().unwrap();
    assert_eq!(rec.payload["project_id"], s.project.to_string());
    assert_eq!(rec.payload["billable"], false);
}
