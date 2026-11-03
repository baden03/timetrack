# TimeTrack — Development Plan: P0 and R1

**Status:** Draft, based on the answers of 3 November 2026.
**Scope:** P0 (Foundations) and R1 (Stand-alone) from [plan.md](plan.md). R2 and later stay at milestone level until the sync spike (S) is done.
**Team:** Baden and Claude build. Doug and Steffen test and give feedback. Hours per week are not fixed, so the schedule is in **weeks of focused work** and is a target, not a promise.
**Target:** R1 feature-complete in about 4 weeks. Acceptance criterion 1 (a real billing month) then runs on top of that.

---

# 1. Decisions taken

| # | Decision | Outcome |
|---|---|---|
| D1 | Shell and UI | Tauri with a TypeScript UI over the Rust core. Risk-checked by task P0-3 before anything depends on it. |
| D2 | License | Apache-2.0 (private use for now; MIT also acceptable). Revisit before any sync engine is adopted. |
| D3 | Repository | One Cargo workspace plus the UI package. |
| D4 | IDs and clocks | UUIDv7-style time-ordered IDs generated on the client, with an origin prefix. Short note written in P0-4. |
| D5 | Summary format | Plain text: client, then project, then category; total hours, then one line per description. Sample output approved by Baden before the summary is built (R1-9). |
| D6 | Menu-bar timer and global shortcut | In R1. |

---

# 2. Repository layout

```
timetrack/
  Cargo.toml                 workspace
  crates/
    core/                    domain rules, IDs, time maths, records, history, summary
    storage/                 storage interface + SQLite implementation
    hub/                     placeholder only (empty binary)
  apps/
    desktop/                 Tauri shell + TypeScript UI
  fixtures/                  shared test vectors (concept §18) as data files
  i18n/                      message catalogs (en)
  docs/
```

`core` has no knowledge of UI, SQLite or network. The desktop talks to it through Tauri commands.

---

# 3. P0 — Foundations (week 1)

| ID | Task | Done when | Size |
|---|---|---|---|
| P0-1 | Workspace, license, CI running `cargo test`, `clippy`, `fmt` and the UI type check | CI is green on an empty change | S |
| P0-2 | Fixtures from concept §18: duration arithmetic, midnight split, the Berlin DST vector (23:00 on 24 Oct to 04:00 on 25 Oct = 1:00 + 5:00, real 6:00) | Fixtures are data files and the core tests read them | S |
| P0-3 | **Tauri risk spike**: a window, a menu-bar item with a live elapsed time, a global shortcut, macOS Keychain access, one Rust command called from TypeScript | Short written result: works, works with caveats, or plan B (see risks) | S |
| P0-4 | Record model: time-ordered IDs, revision, author, device, origin, tombstone, append-only history record. One page on the ID and clock scheme | Create, revise, delete and reconstruct a record with full history in a screen-less test | M |
| P0-5 | Storage interface and SQLite implementation, with schema versioning and migrations | Same tests pass against the interface; two independent databases never collide in IDs | M |
| P0-6 | Message catalog mechanism (English) | No user-visible string is hard-coded; a lint or test enforces it | S |

**P0 exit** (plan.md acceptance): fixtures pass on every change; record lifecycle with history works without a screen; no ID collisions across databases.

---

# 4. R1 — Stand-alone (weeks 2 to 4)

## 4.1 Work packages

| ID | Package | Includes | Depends on | Week |
|---|---|---|---|---|
| R1-1 | First run and identity | Local identity, "stand-alone" choice (architecture 5.9a), preferences (date format, 24 h clock, language) | P0 | 2 |
| R1-2 | Clients, projects, categories | Create, edit, archive; hard delete only in a danger zone | P0-5 | 2 |
| R1-3 | Time entry, start and end | Description required, billable flag, free-form tags (max 40 characters, 10 per entry); midnight split in the client with zone and offsets stored per piece | P0-2, P0-4 | 2 |
| R1-4 | List and editing | Row-level editing (Enter saves, Esc cancels); day and week views over a 60-day window; search over all entries; `hh:mm` display | R1-3 | 2 to 3 |
| R1-5 | Live timer | One running timer; persists across quit and crash; stale-timer prompt on next start | R1-3 | 3 |
| R1-6 | Menu-bar timer and global shortcut | Start, stop, switch project from the menu bar and from a shortcut | R1-5, P0-3 | 3 |
| R1-7 | Duration-only entry | Third entry method | R1-3 | 3 |
| R1-8 | Change history view | Every edit visible with before and after | P0-4 | 3 |
| R1-9 | Monthly summary | Per client, per project, by category: total and descriptions; copy as plain text. Format signed off first | R1-3, D5 | 3 |
| R1-10 | Backup and restore | Export and import of the full database; restore on a fresh install gives identical data | P0-5 | 4 |
| R1-11 | Keyboard pass | Create, edit and finish an entry without the mouse | R1-4 | 4 |
| R1-12 | Hardening | Crash and quit during timer, DST and month-boundary edge cases, offline check, packaging as a signed or ad-hoc macOS app | all | 4 |

## 4.2 Acceptance traceability (plan.md R1)

| Criterion | Covered by |
|---|---|
| 1. A real billing month replaces the Google Doc; summary matches manual figures | R1-9, then dogfooding from end of week 3 |
| 2. Works with network disabled | R1-12 (also: no network code in the desktop for R1) |
| 3. Fixtures pass in the real app, including DST | P0-2, R1-3 (an end-to-end test drives the app through the DST vector) |
| 4. Keyboard-only entry | R1-11 |
| 5. Backup then restore is identical | R1-10 |
| 6. Timer survives quit or crash; stale prompt | R1-5, R1-12 |
| 7. Every edit visible in history | R1-8 |
| 8. Category totals add to project total; every description appears | R1-9 (property test) |

---

# 5. Schedule

| Week | Focus | Checkpoint |
|---|---|---|
| 1 | P0 | P0 exit criteria met; Tauri spike result recorded |
| 2 | R1-1 to R1-4 | Baden can enter and edit start/end entries on his Mac |
| 3 | R1-5 to R1-9 | Timer, menu bar, history and summary. **Baden starts using it alongside the Google Doc** |
| 4 | R1-10 to R1-12 | Release candidate. Doug and Steffen try it on their Macs (stand-alone) and report on the timer and flow |
| 5+ | Real-use month | Compare summary with the manual figures; fix what turns up. This closes R1 criterion 1 |

Doug and Steffen test from week 4 only for the experience (timer, shortcut, summary). They cannot share data until R2.

---

# 6. Working method

- Every package ends with tests plus a short note of what to try by hand.
- The core is built test-first against the fixtures. UI work follows the core, not the other way round.
- Hub items stay out of R1 (hub, sync, rates and values, invoice locks, import). The data model is already sync-ready (plan.md, principle 2), so nothing is added now for R2 except what P0-4 requires.
- Claude drafts, Baden reviews at each week's checkpoint and decides open points on the spot.

---

# 7. Risks

| Risk | Effect | Response |
|---|---|---|
| Tauri menu-bar item or global shortcut works poorly | R1-6 slips; Doug's main need | P0-3 finds out in week 1. Plan B: a small native Swift helper for the menu bar, or an in-window timer first and the menu bar before Doug joins |
| Time and zone edge cases | Wrong dates in the summary | Fixtures first, property tests for splits, offsets stored per piece |
| Part-time hours make four weeks unrealistic | Slips | Cut order if needed: R1-7 (duration only), then R1-11 polish, then search. Never cut history, backup or the summary |
| Sync-ready model over-engineered | Delays R1 | P0-4 limited to what plan.md lists |
| Unsigned macOS app blocked by Gatekeeper on Doug's and Steffen's Macs | Testers cannot open it | Ad-hoc signing and a short install note; Apple developer signing only if needed |

---

# 8. After R1

1. Write the scenario tests for the sync spike (S) while the real-use month runs.
2. Run the spike (architecture §7.2) and decide the sync approach.
3. Break R2 into tasks the same way as this document.
