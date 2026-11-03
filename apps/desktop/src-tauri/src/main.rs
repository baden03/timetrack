//! P0-3 risk spike: proves the pieces release 1 depends on work with Tauri on
//! macOS — a menu-bar item showing live elapsed time, a global shortcut,
//! Keychain access, and a Rust command called from TypeScript. Throwaway code;
//! the real shell replaces it in release 1.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::tray::TrayIconBuilder;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use timetrack_core::id::{IdGenerator, Origin};
use timetrack_core::time::format_hhmm;

type Timer = Arc<Mutex<Option<Instant>>>;

#[derive(Serialize)]
struct Ping {
    hhmm: String,
    id: String,
    keychain_ok: bool,
}

fn keychain_roundtrip() -> Result<(), String> {
    let entry = keyring::Entry::new("de.twinpictures.timetrack.spike", "spike")
        .map_err(|e| e.to_string())?;
    entry.set_password("spike-secret").map_err(|e| e.to_string())?;
    let got = entry.get_password().map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())?;
    if got == "spike-secret" {
        Ok(())
    } else {
        Err("value mismatch".into())
    }
}

#[tauri::command]
fn core_ping(minutes: i64) -> Ping {
    let ids = IdGenerator::new(Origin::random());
    Ping {
        hhmm: format_hhmm(minutes),
        id: ids.next_id().to_string(),
        keychain_ok: keychain_roundtrip().is_ok(),
    }
}

fn toggle(timer: &Timer, app: &AppHandle) {
    let mut guard = timer.lock().unwrap();
    *guard = match *guard {
        Some(_) => None,
        None => Some(Instant::now()),
    };
    let running = guard.is_some();
    drop(guard);
    eprintln!("spike: timer {}", if running { "started" } else { "stopped" });
    set_title(app, if running { Some(0) } else { None });
}

fn set_title(app: &AppHandle, seconds: Option<u64>) {
    let Some(tray) = app.tray_by_id("main") else {
        eprintln!("spike: tray missing");
        return;
    };
    let title = seconds.map(|s| format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60));
    match tray.set_title(title.as_deref()) {
        Ok(()) => eprintln!("spike: tray title = {title:?}"),
        Err(e) => eprintln!("spike: set_title failed: {e}"),
    }
}

fn main() {
    let timer: Timer = Arc::new(Mutex::new(None));
    let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT);

    let handler_timer = timer.clone();
    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                eprintln!("spike: global shortcut pressed");
                toggle(&handler_timer, app);
            }
        })
        .build();

    tauri::Builder::default()
        .plugin(plugin)
        .invoke_handler(tauri::generate_handler![core_ping])
        .setup(move |app| {
            let icon = app.default_window_icon().cloned().ok_or("no icon")?;
            TrayIconBuilder::with_id("main")
                .icon(icon)
                .title("TT")
                .build(app)?;
            eprintln!("spike: tray created");

            match app.global_shortcut().register(shortcut) {
                Ok(()) => eprintln!("spike: shortcut registered"),
                Err(e) => eprintln!("spike: shortcut registration failed: {e}"),
            }
            eprintln!("spike: keychain roundtrip = {:?}", keychain_roundtrip());

            let handle = app.handle().clone();
            let ticker = timer.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                let started = *ticker.lock().unwrap();
                if let Some(start) = started {
                    set_title(&handle, Some(start.elapsed().as_secs()));
                }
            });

            if std::env::var("TT_SPIKE_AUTOTOGGLE").is_ok() {
                let handle = app.handle().clone();
                let auto = timer.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(2));
                    toggle(&auto, &handle);
                    std::thread::sleep(Duration::from_secs(4));
                    toggle(&auto, &handle);
                    std::thread::sleep(Duration::from_secs(1));
                    eprintln!("spike: autotoggle done");
                    handle.exit(0);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
