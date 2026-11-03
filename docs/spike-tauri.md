# Tauri risk spike (P0-3)

**Date:** 3 November 2026. **Code:** `apps/desktop/src-tauri/src/main.rs` (throwaway; replaced in R1).
**Versions:** Tauri 2, `tauri-plugin-global-shortcut` 2, `keyring` 3 (Apple native), Rust 1.99, macOS 15.7.

**Result: works.** No fallback to a Swift helper is needed for R1-6.

| Need | Result | Evidence |
|---|---|---|
| Rust core called from the shell | Works | The app builds against `timetrack-core`; the `core_ping` command compiles and is registered. The TypeScript call was not observed in the window. |
| Menu-bar item with live elapsed time | Works | A tray icon was created and its title updated each second (`0:00:00` to `0:00:03`), then cleared on stop. Seen in the app's log; not looked at on screen. |
| Global shortcut (Cmd+Shift+T) | Registers | Registration succeeded. A physical key press was not tested; the handler shares the toggle code that the automatic test exercised. |
| Keychain access | Works | Write, read and delete round trip returned the same value with no prompt. |
| Front-end toolchain | Works | `pnpm` typecheck and Vite build pass. |

## Caveats to carry into R1-6

- Test the real shortcut press and the on-screen menu-bar text by hand once on a Mac.
- The tray title is plain text. A running-state icon change needs a second icon asset.
- The desktop crate has its own Cargo workspace (excluded from the root one) so core checks stay fast. CI for the shell will be added with R1-1.
- An unsigned build may be blocked by Gatekeeper on other Macs (risk in the development plan).
