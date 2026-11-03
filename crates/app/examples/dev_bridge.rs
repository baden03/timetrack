//! Development only: serves the application service over local HTTP so the
//! front end can be developed and tested in an ordinary browser, without the
//! Tauri shell. Usage: `cargo run -p timetrack-app --example dev_bridge -- <db path> [port]`.

use serde_json::{json, Value};
use timetrack_app::{system_clock, App};
use timetrack_storage::SqliteStore;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("database path");
    let port = args.next().unwrap_or_else(|| "1421".into());
    let store = SqliteStore::open(std::path::Path::new(&path)).expect("open database");
    let zone = std::env::var("TT_ZONE").unwrap_or_else(|_| "Europe/Berlin".into());
    let mut app = App::open(store, &zone, system_clock()).expect("open app");
    let server = tiny_http::Server::http(format!("127.0.0.1:{port}")).expect("listen");
    eprintln!("dev bridge on http://127.0.0.1:{port}");

    let cors = |r: tiny_http::Response<std::io::Cursor<Vec<u8>>>| {
        r.with_header(
            "Access-Control-Allow-Origin: *"
                .parse::<tiny_http::Header>()
                .unwrap(),
        )
        .with_header(
            "Access-Control-Allow-Headers: content-type"
                .parse::<tiny_http::Header>()
                .unwrap(),
        )
        .with_header(
            "Content-Type: application/json"
                .parse::<tiny_http::Header>()
                .unwrap(),
        )
    };
    for mut request in server.incoming_requests() {
        if request.method() == &tiny_http::Method::Options {
            let _ = request.respond(cors(tiny_http::Response::from_string("")));
            continue;
        }
        let mut body = String::new();
        let _ = request.as_reader().read_to_string(&mut body);
        let call: Value = serde_json::from_str(&body).unwrap_or(json!({}));
        let command = call["command"].as_str().unwrap_or("");
        let (status, out) = match app.dispatch(command, &call["args"]) {
            Ok(v) => (200, v),
            Err(e) => (400, json!({"code": e.code()})),
        };
        let _ = request.respond(cors(
            tiny_http::Response::from_string(out.to_string()).with_status_code(status),
        ));
    }
}
