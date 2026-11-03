# Development

## Tools
- Rust stable via `rustup` (pinned by `rust-toolchain.toml`)
- Node 22 or newer (`.nvmrc` says 26) and `pnpm`

## Everyday commands
```bash
scripts/check.sh                              # format, clippy, tests (same as CI)
cargo test --workspace                        # core and storage tests, no screen needed
cd apps/desktop && pnpm install && pnpm dev   # front-end only
cd apps/desktop/src-tauri && cargo build      # desktop shell (needs `pnpm dev` running)
```

## Layout
- `crates/core`: domain rules, IDs, time maths, records, message catalog
- `crates/storage`: storage interface and SQLite implementation
- `crates/app`: application service (commands, validation); the only layer the interface talks to
- `crates/hub`: placeholder until release 2
- `apps/desktop`: Tauri shell and React + TypeScript UI
- `fixtures/`: shared test vectors from the concept (section 18)
- `i18n/`: message catalogs

## Working on the UI without the Tauri shell
The front end falls back to a local HTTP bridge when it is not inside Tauri, so it can be driven in an ordinary browser:
```bash
cargo run -p timetrack-app --example dev_bridge -- /tmp/tt-dev.sqlite   # service on 127.0.0.1:1421
cd apps/desktop && pnpm dev                                              # UI on http://localhost:1420
```
The shell stores its data in the app data directory; set `TT_DATA_DIR` to use a throwaway folder.

## Building the macOS app
```bash
cd apps/desktop && pnpm install && pnpm tauri build --bundles app
```
The app is written to `apps/desktop/src-tauri/target/release/bundle/macos/TimeTrack.app` and is signed ad hoc (no Apple developer account needed; see `docs/install-macos.md` for what that means for testers). To share it:
```bash
ditto -c -k --keepParent apps/desktop/src-tauri/target/release/bundle/macos/TimeTrack.app TimeTrack.zip
```

## Checks
`scripts/check.sh` runs everything CI runs: formatting, clippy, all Rust tests, the desktop shell's tests, the UI type check and tests, and `scripts/no-network.sh`, which keeps release 1 free of network use.
