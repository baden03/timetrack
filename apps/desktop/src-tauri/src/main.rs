//! Desktop shell: opens the local database and forwards the interface's
//! commands to the application service. No business rules live here.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use serde_json::Value;
use tauri::Manager;
use timetrack_app::{system_clock, App};
use timetrack_storage::SqliteStore;

struct AppState(Mutex<App<SqliteStore>>);

#[derive(Serialize)]
struct CommandError {
    code: String,
}

#[tauri::command]
fn call(
    state: tauri::State<AppState>,
    command: String,
    args: Value,
) -> Result<Value, CommandError> {
    let mut app = state.0.lock().map_err(|_| CommandError {
        code: "storage".into(),
    })?;
    app.dispatch(&command, &args).map_err(|e| CommandError {
        code: e.code().into(),
    })
}

fn data_dir(app: &tauri::App) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Development and tests point this at a throwaway directory.
    if let Ok(dir) = std::env::var("TT_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    Ok(app.path().app_data_dir()?)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![call])
        .setup(|app| {
            let dir = data_dir(app)?;
            std::fs::create_dir_all(&dir)?;
            let store = SqliteStore::open(&dir.join("timetrack.sqlite"))?;
            let zone = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".into());
            let service = App::open(store, &zone, system_clock())?;
            app.manage(AppState(Mutex::new(service)));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
