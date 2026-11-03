#!/usr/bin/env bash
# Runs the same checks as CI. Usage: scripts/check.sh
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
if [ -f apps/desktop/package.json ] && [ -d apps/desktop/node_modules ]; then
  (cd apps/desktop && pnpm run typecheck && pnpm test)
  (cd apps/desktop/src-tauri && cargo test)
fi
scripts/no-network.sh
