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
    let client = app.create_client("Acme", None).unwrap();
    let category = app.create_category("Development").unwrap();
    let project = app
        .create_project(ProjectInput {
            client_id: client.id,
            name: "Website".into(),
            description: String::new(),
            billable_default: true,
            category_ids: vec![category.id.clone()],
            color: None,
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
            category_id: Some(s.category.clone()),
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
        category_id: None,
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
        category_id: Some(s.category.clone()),
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
        let client = app.create_client("Acme", None).unwrap();
        let category = app.create_category("Dev").unwrap();
        let project = app
            .create_project(ProjectInput {
                client_id: client.id,
                name: "Site".into(),
                description: String::new(),
                billable_default: true,
                category_ids: vec![category.id.clone()],
                color: None,
            })
            .unwrap();
        app.start_timer(StartTimer {
            project_id: project.id.clone(),
            category_id: Some(category.id.clone()),
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
        "timer_too_short"
    );
    assert!(s.app.timer_state().unwrap().timer.is_some());
    // It can still be discarded.
    s.app.discard_timer().unwrap();
    assert!(s.app.timer_state().unwrap().timer.is_none());
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
            category_id: Some(s.category.clone()),
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

#[test]
fn a_timer_can_start_without_a_category_and_get_one_at_stop() {
    let mut s = setup("2026-11-03T09:00:00Z");
    s.app
        .start_timer(StartTimer {
            project_id: s.project.clone(),
            category_id: None,
            description: String::new(),
            tags: vec![],
            billable: None,
        })
        .unwrap();
    advance(&s, "2026-11-03T10:00:00Z");
    let mut stop_with = stop("Chosen at stop", None);
    stop_with.category_id = Some(Some(s.category.clone()));
    let saved = s.app.stop_timer(stop_with).unwrap();
    assert_eq!(
        saved.pieces[0].payload["category_id"],
        s.category.to_string()
    );

    // And with no category at all.
    s.app
        .start_timer(StartTimer {
            project_id: s.project.clone(),
            category_id: None,
            description: String::new(),
            tags: vec![],
            billable: None,
        })
        .unwrap();
    advance(&s, "2026-11-03T11:00:00Z");
    let saved = s.app.stop_timer(stop("No category", None)).unwrap();
    assert!(saved.pieces[0].payload["category_id"].is_null());
}

fn entry_on(s: &mut Setup, project: &Id, date: &str, start: &str, end: &str) {
    s.app
        .create_entry(timetrack_app::NewEntry {
            project_id: project.clone(),
            category_id: None,
            description: "work".into(),
            billable: None,
            tags: vec![],
            timing: timetrack_app::Timing::Interval {
                date: date.into(),
                start: start.into(),
                end: end.into(),
            },
            timezone: None,
        })
        .unwrap();
}

#[test]
fn dashboard_orders_projects_by_recent_use_and_overview_totals() {
    let mut s = setup("2026-11-03T09:00:00Z"); // a Tuesday
    let client = s.app.list("client", false).unwrap().remove(0);
    advance(&s, "2026-11-03T09:05:00Z");
    let second = s
        .app
        .create_project(ProjectInput {
            client_id: client.id.clone(),
            name: "Blog".into(),
            description: String::new(),
            billable_default: true,
            category_ids: vec![],
            color: None,
        })
        .unwrap();
    let names = |s: &Setup| -> Vec<String> {
        s.app
            .dashboard()
            .unwrap()
            .iter()
            .map(|c| c.project.payload["name"].as_str().unwrap().to_owned())
            .collect()
    };
    // A new project is the most recent until time is recorded on the older one.
    assert_eq!(names(&s), ["Blog", "Website"]);
    advance(&s, "2026-11-03T12:00:00Z");
    let website = s.project.clone();
    entry_on(&mut s, &website, "2026-11-03", "09:00", "10:30");
    assert_eq!(names(&s), ["Website", "Blog"]);
    assert_eq!(s.app.dashboard().unwrap()[0].client_name, "Acme");

    // A running timer puts its project first.
    advance(&s, "2026-11-03T12:30:00Z");
    s.app
        .start_timer(StartTimer {
            project_id: second.id.clone(),
            category_id: None,
            description: String::new(),
            tags: vec![],
            billable: None,
        })
        .unwrap();
    let cards = s.app.dashboard().unwrap();
    assert!(cards[0].running && cards[0].project.id == second.id);
    s.app.discard_timer().unwrap();

    // Archived projects are not shown.
    s.app
        .set_project_status(&second.id, timetrack_core::model::Status::Archived)
        .unwrap();
    assert_eq!(names(&s), ["Website"]);

    // Today (Tue 3 Nov), this week, this month, all time.
    entry_on(&mut s, &website, "2026-11-02", "09:00", "10:00"); // Monday: 60
    entry_on(&mut s, &website, "2026-11-01", "09:00", "10:00"); // Sunday of last week: 60
    entry_on(&mut s, &website, "2026-10-05", "09:00", "11:00"); // October: 120
    let o = s.app.project_overview(&website).unwrap();
    assert_eq!(
        (
            o.today_minutes,
            o.week_minutes,
            o.month_minutes,
            o.total_minutes
        ),
        (90, 150, 210, 330)
    );
    assert_eq!(s.app.project_overview(&second.id).unwrap().total_minutes, 0);
}

#[test]
fn the_display_start_is_exact_while_the_entry_start_is_to_the_minute() {
    let mut s = setup("2026-11-03T09:00:42Z");
    let rec = start(&mut s);
    assert_eq!(rec.payload["start_at"], "2026-11-03T09:00:00Z");
    assert_eq!(rec.payload["exact_start_at"], "2026-11-03T09:00:42Z");
    // A timer stored before this field existed still loads.
    let mut old = rec.payload.clone();
    old.as_object_mut().unwrap().remove("exact_start_at");
    let parsed: timetrack_core::model::Timer = serde_json::from_value(old).unwrap();
    assert!(parsed.exact_start_at.is_none());
}

#[test]
fn running_notes_become_bullet_points_of_the_entry() {
    let mut s = setup("2026-11-03T09:00:00Z");
    start(&mut s);
    s.app.add_timer_note("  Setup folder   structure ").unwrap();
    s.app.add_timer_note("Add placeholder files").unwrap();
    assert_eq!(
        s.app.add_timer_note("   ").unwrap_err().code(),
        "description_required"
    );
    advance(&s, "2026-11-03T10:00:00Z");

    // The text typed at the stop is the last bullet.
    let saved = s
        .app
        .stop_timer(stop("Update placeholder text with the client's text", None))
        .unwrap();
    assert_eq!(
        saved.pieces[0].payload["description"],
        "- Setup folder structure\n- Add placeholder files\n- Update placeholder text with the client's text"
    );
    assert!(s.app.timer_state().unwrap().timer.is_none());
}

#[test]
fn notes_alone_are_enough_and_one_piece_stays_plain() {
    let mut s = setup("2026-11-03T09:00:00Z");
    start(&mut s);
    s.app.add_timer_note("Only note").unwrap();
    advance(&s, "2026-11-03T09:30:00Z");
    // Nothing typed at the stop: the note alone is the description, without a bullet.
    let saved = s.app.stop_timer(stop("", None)).unwrap();
    assert_eq!(saved.pieces[0].payload["description"], "Only note");

    // With no notes and no text, the stop is refused and the timer stays.
    start(&mut s);
    advance(&s, "2026-11-03T10:00:00Z");
    assert_eq!(
        s.app.stop_timer(stop("  ", None)).unwrap_err().code(),
        "description_required"
    );
    assert!(s.app.timer_state().unwrap().timer.is_some());
}

#[test]
fn a_note_can_be_removed_and_survives_a_restart() {
    let dir = std::env::temp_dir().join(format!("tt-notes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("db.sqlite");
    let now: Now = Arc::new(Mutex::new(at("2026-11-03T09:00:00Z")));
    let project = {
        let mut app = App::open(
            SqliteStore::open(&path).unwrap(),
            "Europe/Berlin",
            clock(&now),
        )
        .unwrap();
        let client = app.create_client("Acme", None).unwrap();
        let project = app
            .create_project(ProjectInput {
                client_id: client.id,
                name: "Site".into(),
                description: String::new(),
                billable_default: true,
                category_ids: vec![],
                color: None,
            })
            .unwrap();
        app.start_timer(StartTimer {
            project_id: project.id.clone(),
            category_id: None,
            description: String::new(),
            tags: vec![],
            billable: None,
        })
        .unwrap();
        app.add_timer_note("one").unwrap();
        app.add_timer_note("two").unwrap();
        app.add_timer_note("three").unwrap();
        app.remove_timer_note(1).unwrap();
        assert_eq!(app.remove_timer_note(7).unwrap_err().code(), "not_found");
        project.id
    }; // dropped without stopping
    let _ = project;
    let app = App::open(
        SqliteStore::open(&path).unwrap(),
        "Europe/Berlin",
        clock(&now),
    )
    .unwrap();
    let state = app.timer_state().unwrap();
    assert_eq!(
        state.timer.unwrap().payload["notes"],
        serde_json::json!(["one", "three"])
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_last_used_category_is_proposed_and_kept_for_quick_start() {
    let mut s = setup("2026-11-03T09:00:00Z");
    let other = s.app.create_category("Meetings").unwrap();
    let client = s.app.list("client", false).unwrap().remove(0);
    let project = s
        .app
        .create_project(ProjectInput {
            client_id: client.id,
            name: "Both".into(),
            description: String::new(),
            billable_default: true,
            category_ids: vec![s.category.clone(), other.id.clone()],
            color: None,
        })
        .unwrap();
    advance(&s, "2026-11-03T12:00:00Z");
    let entry = |s: &mut Setup, cat: Option<Id>, start: &str, end: &str| {
        s.app
            .create_entry(timetrack_app::NewEntry {
                project_id: project.id.clone(),
                category_id: cat,
                description: "x".into(),
                billable: None,
                tags: vec![],
                timing: timetrack_app::Timing::Interval {
                    date: "2026-11-03".into(),
                    start: start.into(),
                    end: end.into(),
                },
                timezone: None,
            })
            .unwrap();
    };
    // Nothing recorded yet: no proposal.
    assert_eq!(
        s.app
            .suggest(&project.id, "2026-11-03")
            .unwrap()
            .category_id,
        None
    );
    let dev = s.category.clone();
    entry(&mut s, Some(dev.clone()), "09:00", "10:00");
    advance(&s, "2026-11-03T12:10:00Z");
    entry(&mut s, Some(other.id.clone()), "10:00", "11:00");
    advance(&s, "2026-11-03T12:20:00Z");
    // A later entry without a category does not erase the proposal.
    entry(&mut s, None, "11:00", "12:00");
    assert_eq!(
        s.app
            .suggest(&project.id, "2026-11-03")
            .unwrap()
            .category_id,
        Some(other.id.clone())
    );

    // Quick start takes the newest entry's project and that project's last used category.
    let rec = s.app.quick_start_timer().unwrap();
    assert_eq!(rec.payload["category_id"], other.id.to_string());
    let state = s.app.timer_state().unwrap();
    assert_eq!(state.suggested_category_id, Some(other.id.clone()));
    s.app.discard_timer().unwrap();

    // An archived category is skipped; the next older one is proposed instead.
    s.app
        .set_category_status(&other.id, timetrack_core::model::Status::Archived)
        .unwrap();
    assert_eq!(
        s.app
            .suggest(&project.id, "2026-11-03")
            .unwrap()
            .category_id,
        Some(dev)
    );
}
