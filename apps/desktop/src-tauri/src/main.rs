//! Desktop shell: opens the local database, forwards the interface's
//! commands to the application service, and provides the menu-bar timer and
//! the global shortcut. No business rules live here.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, RunEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use timetrack_app::{system_clock, App};
use timetrack_storage::SqliteStore;

struct AppState(Mutex<App<SqliteStore>>);

#[derive(Serialize)]
struct CommandError {
    code: String,
}

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
    let result = app.dispatch(&command, &args).map_err(|e| CommandError {
        code: e.code().into(),
    });
    drop(app);
    if result.is_ok() && command.contains("timer") {
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

/// Start of the running timer, if any.
fn running_since(app: &AppHandle) -> Option<DateTime<Utc>> {
    let state = app.state::<AppState>();
    let mut service = state.0.lock().ok()?;
    let timer = service.dispatch("timer_state", &json!({})).ok()?;
    timer["timer"]["payload"]["start_at"]
        .as_str()?
        .parse::<DateTime<Utc>>()
        .ok()
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
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

fn elapsed_title(since: DateTime<Utc>) -> String {
    let total = (Utc::now() - since).num_seconds().max(0);
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
        .invoke_handler(tauri::generate_handler![call])
        .setup(move |app| {
            let dir = data_dir(app)?;
            std::fs::create_dir_all(&dir)?;
            let store = SqliteStore::open(&dir.join("timetrack.sqlite"))?;
            let zone = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".into());
            let service = App::open(store, &zone, system_clock())?;
            app.manage(AppState(Mutex::new(service)));

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show", "Show TimeTrack", true, None::<&str>)?,
                    &MenuItem::with_id(app, "toggle", "Start / stop timer", true, None::<&str>)?,
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

            // The menu-bar title follows the stored timer, so a timer started
            // anywhere (window, shortcut, a previous run) shows up.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut shown: Option<String> = None;
                loop {
                    let title = running_since(&handle).map(elapsed_title);
                    if title != shown {
                        if let Some(tray) = handle.tray_by_id("main") {
                            let result = tray.set_title(title.as_deref());
                            if std::env::var("TT_DEBUG_TRAY").is_ok() {
                                eprintln!("tray title {title:?}: {result:?}");
                            }
                        }
                        shown = title;
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    built.run(|_app, event| {
        // Closing the window keeps the menu-bar timer alive; Quit exits.
        if let RunEvent::ExitRequested { code: None, api, .. } = event {
            api.prevent_exit();
        }
    });
}
