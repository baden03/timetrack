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
use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::Wry;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use timetrack_app::{system_clock, App};
use timetrack_core::i18n::Catalog;
use timetrack_storage::SqliteStore;

/// The service and the folder the data lives in.
struct AppState(Mutex<App<SqliteStore>>, PathBuf);

/// The tray menu entries, kept so their text can follow the language chosen
/// in Settings (and the running state, for the start/stop entry).
struct TrayMenu {
    show: MenuItem<Wry>,
    toggle: MenuItem<Wry>,
    quit: MenuItem<Wry>,
    language: Mutex<String>,
}

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
    if result.is_ok()
        && ["update_settings", "restore", "finish_first_run"].contains(&command.as_str())
    {
        refresh_tray_language(&app_handle);
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

const REPOSITORY_URL: &str = "https://github.com/baden03/timetrack";

/// The interface language chosen in Settings (English if it cannot be read).
fn settings_language(app: &AppHandle) -> String {
    let state = app.state::<AppState>();
    let Ok(mut service) = state.0.lock() else {
        return "en".into();
    };
    let Ok(settings) = service.dispatch("state", &json!({})) else {
        return "en".into();
    };
    settings["settings"]["payload"]["language"]
        .as_str()
        .unwrap_or("en")
        .to_owned()
}

/// The application menu, with an About panel that says what TimeTrack is and
/// where its source lives. The text follows the language at start-up.
fn build_app_menu<M: Manager<Wry>>(app: &M, language: &str) -> tauri::Result<Menu<Wry>> {
    let tagline = Catalog::for_language(language)
        .text("ui.about.tagline")
        .to_owned();
    let about = AboutMetadata {
        name: Some("TimeTrack".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        copyright: Some("© 2026 Twinpictures".into()),
        comments: Some(tagline.clone()),
        credits: Some(format!("{tagline}\n{REPOSITORY_URL}")),
        website: Some(REPOSITORY_URL.into()),
        website_label: Some("GitHub".into()),
        license: Some("Apache-2.0".into()),
        ..Default::default()
    };
    // Quit is our own item: the window only hides when closed, and the app
    // must still end when the person asks it to.
    let quit = MenuItem::with_id(app, "app-quit", "Quit TimeTrack", true, Some("CmdOrCtrl+Q"))?;
    let app_menu = Submenu::with_items(
        app,
        "TimeTrack",
        true,
        &[
            &PredefinedMenuItem::about(app, Some("About TimeTrack"), Some(about))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    Menu::with_items(app, &[&app_menu, &edit, &window])
}

/// Reads the interface language from the settings and retitles the tray menu.
fn refresh_tray_language(app: &AppHandle) {
    let language = settings_language(app);
    let menu = app.state::<TrayMenu>();
    let catalog = Catalog::for_language(&language);
    let _ = menu.show.set_text(catalog.text("tray.show"));
    let _ = menu.quit.set_text(catalog.text("tray.quit"));
    let mut current = match menu.language.lock() {
        Ok(current) => current,
        Err(_) => return,
    };
    *current = language;
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
        // Set before the window exists, so the menu bar is ours from the start;
        // setup then rebuilds it once the language is known.
        .menu(|handle| build_app_menu(handle, "en"))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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

            let show_item = MenuItem::with_id(app, "show", "Show TimeTrack", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Start timer", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit TimeTrack", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &toggle_item, &quit_item])?;
            app.manage(TrayMenu {
                show: show_item,
                toggle: toggle_item,
                quit: quit_item,
                language: Mutex::new("en".into()),
            });
            refresh_tray_language(app.handle());

            let language = settings_language(app.handle());
            app.set_menu(build_app_menu(app, &language)?)?;
            app.on_menu_event(|app, event| {
                if event.id().as_ref() == "app-quit" {
                    app.exit(0);
                }
            });
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
                let mut menu_shown: Option<(bool, String)> = None;
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
                    let menu = handle.state::<TrayMenu>();
                    let language = menu.language.lock().map(|l| l.clone()).unwrap_or_default();
                    if menu_shown.as_ref() != Some(&(running, language.clone())) {
                        let catalog = Catalog::for_language(&language);
                        let key = if running { "tray.stop" } else { "tray.start" };
                        let _ = menu.toggle.set_text(catalog.text(key));
                        menu_shown = Some((running, language));
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
