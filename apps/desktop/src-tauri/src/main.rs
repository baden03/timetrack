//! Desktop shell: opens the local database, forwards the interface's
//! commands to the application service, and provides the menu-bar timer and
//! the global shortcut. No business rules live here.
//!
//! The stored timer is the single source of truth. The window and the menu
//! bar both read it, both count elapsed time from the same stored start, and
//! every change to it (from the window, the shortcut or the tray menu) sends
//! one `timer-changed` event so the window reloads everything that shows it.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use timetrack_app::{system_clock, App};
use timetrack_storage::SqliteStore;

/// The service and the folder the data lives in.
struct AppState(Mutex<App<SqliteStore>>, PathBuf);

#[derive(Serialize)]
struct CommandError {
    code: String,
}

/// Commands that change the timer (reads such as `timer_state` must not
/// announce a change, or the window would reload itself forever).
const TIMER_CHANGES: [&str; 5] = [
    "start_timer",
    "quick_start_timer",
    "update_timer",
    "stop_timer",
    "discard_timer",
];

#[tauri::command]
fn call(
    app_handle: AppHandle,
    state: tauri::State<AppState>,
    command: String,
    args: Value,
) -> Result<Value, CommandError> {
    let mut app = state.0.lock().map_err(|_| CommandError {
        code: "storage".into(),
    })?;
    let mut args = args;
    if command == "restore" {
        // Before a restore the current data is copied here, so it can be undone.
        let dir = state.1.join("backups");
        let _ = std::fs::create_dir_all(&dir);
        let stamp = Utc::now().format("%Y%m%d-%H%M%S");
        args["safety_path"] = json!(dir.join(format!("before-restore-{stamp}.sqlite")));
    }
    let result = app.dispatch(&command, &args).map_err(|e| CommandError {
        code: e.code().into(),
    });
    drop(app);
    if result.is_ok() && (TIMER_CHANGES.contains(&command.as_str()) || command == "restore") {
        let _ = app_handle.emit("timer-changed", ());
    }
    result
}

fn data_dir(app: &tauri::App) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Development and tests point this at a throwaway directory.
    if let Ok(dir) = std::env::var("TT_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    Ok(app.path().app_data_dir()?)
}

/// When the running timer actually started, if one is running.
fn running_since(app: &AppHandle) -> Option<DateTime<Utc>> {
    let state = app.state::<AppState>();
    let mut service = state.0.lock().ok()?;
    let timer = service.dispatch("timer_state", &json!({})).ok()?;
    let payload = &timer["timer"]["payload"];
    payload["exact_start_at"]
        .as_str()
        .or_else(|| payload["start_at"].as_str())?
        .parse::<DateTime<Utc>>()
        .ok()
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// The shortcut and the menu entry: start a timer on the last project, or,
/// if one is running, ask the window to confirm the description and stop.
fn toggle_timer(app: &AppHandle) {
    if running_since(app).is_some() {
        show_window(app);
        let _ = app.emit("timer-stop-request", ());
        return;
    }
    let started = {
        let state = app.state::<AppState>();
        let mut service = state.0.lock().expect("service lock");
        service.dispatch("quick_start_timer", &json!({})).is_ok()
    };
    if started {
        let _ = app.emit("timer-changed", ());
    } else {
        // Nothing to repeat yet: let the person choose in the window.
        show_window(app);
    }
}

/// `h:mm:ss`, from the same start and with the same formula as the window.
fn elapsed_title(since: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let total = (now - since).num_seconds().max(0);
    format!("{}:{:02}:{:02}", total / 3600, total / 60 % 60, total % 60)
}

fn main() {
    let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT);
    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_timer(app);
            }
        })
        .build();

    let built = tauri::Builder::default()
        .plugin(plugin)
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![call])
        // Closing the window hides it: the timer, the menu bar and the
        // shortcut keep working, and the window can be shown again.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            let dir = data_dir(app)?;
            std::fs::create_dir_all(&dir)?;
            let store = SqliteStore::open(&dir.join("timetrack.sqlite"))?;
            let zone = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".into());
            let service = App::open(store, &zone, system_clock())?;
            app.manage(AppState(Mutex::new(service), dir));

            let toggle_item = MenuItem::with_id(app, "toggle", "Start timer", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show", "Show TimeTrack", true, None::<&str>)?,
                    &toggle_item,
                    &MenuItem::with_id(app, "quit", "Quit TimeTrack", true, None::<&str>)?,
                ],
            )?;
            let icon = app.default_window_icon().cloned().ok_or("no window icon")?;
            TrayIconBuilder::with_id("main")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_window(app),
                    "toggle" => toggle_timer(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            if let Err(e) = app.global_shortcut().register(shortcut) {
                eprintln!("global shortcut not registered: {e}");
            }

            // The menu-bar title and menu follow the stored timer, so a timer
            // started anywhere (window, shortcut, a previous run) shows up.
            // Ticks fall on the whole seconds of the elapsed time, as in the
            // window, so both change at the same moment.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut shown: Option<String> = None;
                let mut was_running: Option<bool> = None;
                loop {
                    let since = running_since(&handle);
                    let title = since.map(|s| elapsed_title(s, Utc::now()));
                    if title != shown {
                        if let Some(tray) = handle.tray_by_id("main") {
                            let result = tray.set_title(title.as_deref());
                            if std::env::var("TT_DEBUG_TRAY").is_ok() {
                                eprintln!("tray title {title:?}: {result:?}");
                            }
                        }
                        shown = title;
                    }
                    let running = since.is_some();
                    if was_running != Some(running) {
                        let _ = toggle_item.set_text(if running {
                            "Stop timer"
                        } else {
                            "Start timer"
                        });
                        was_running = Some(running);
                    }
                    let wait = match since {
                        Some(s) => {
                            let into_second = (Utc::now() - s).num_milliseconds().rem_euclid(1000);
                            (1000 - into_second).clamp(20, 1000) as u64
                        }
                        None => 250,
                    };
                    std::thread::sleep(Duration::from_millis(wait));
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    built.run(|app, event| match event {
        // The window is only hidden when closed; keep running for the menu bar.
        RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        // Clicking the Dock icon brings the window back.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => show_window(app),
        _ => {
            let _ = app;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    /// The same vectors are tested for `formatElapsed` in src/time.test.ts,
    /// so the window and the menu bar show identical text.
    #[test]
    fn elapsed_text_matches_the_windows() {
        let start = at("2026-11-03T09:00:00Z");
        assert_eq!(elapsed_title(start, at("2026-11-03T09:00:00Z")), "0:00:00");
        assert_eq!(
            elapsed_title(start, at("2026-11-03T09:00:59.999Z")),
            "0:00:59"
        );
        assert_eq!(elapsed_title(start, at("2026-11-03T10:02:05Z")), "1:02:05");
        assert_eq!(elapsed_title(start, at("2026-11-04T12:00:00Z")), "27:00:00");
        // A clock a little behind the start never shows a negative time.
        assert_eq!(elapsed_title(start, at("2026-11-03T08:59:59Z")), "0:00:00");
    }

    #[test]
    fn reads_never_announce_a_timer_change() {
        assert!(!TIMER_CHANGES.contains(&"timer_state"));
        assert!(!TIMER_CHANGES.contains(&"dashboard"));
        for command in [
            "start_timer",
            "stop_timer",
            "discard_timer",
            "quick_start_timer",
            "update_timer",
        ] {
            assert!(TIMER_CHANGES.contains(&command));
        }
    }
}
