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
- `crates/hub`: placeholder until release 2
- `apps/desktop`: Tauri shell and TypeScript UI (spike only so far)
- `fixtures/`: shared test vectors from the concept (section 18)
- `i18n/`: message catalogs
