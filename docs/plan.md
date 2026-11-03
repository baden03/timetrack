# TimeTrack — Phased Rollout Plan

**Status:** Draft. The scope changes in section 2 were confirmed on 3 November 2026. The rest is not yet approved.
**Inputs:** [README.md](../README.md) (Concept v2), [architecture.md](architecture.md), and the scoping answers of 2 November 2026 (section 1).
**Sizes** are relative (S, M, L, XL), not durations. No date or team size has been given, so this plan sequences by dependency and value. Turning sizes into a calendar needs the answer to open decision 8.

---

# 1. What the first users need

| Question | Answer | What it means for the plan |
|---|---|---|
| Who uses the first release? | Baden, Doug and Steffen | Three trusted users. No unrelated teams, no multi-tenant concerns. |
| What must it replace? | The Google Docs time log. The goals are to rely less on Google Workspace and to stop asking collaborators for their hours before writing invoices. | Baden alone is served by the first release. The second pain is only removed once collaborators' hours reach Baden automatically (release 2). |
| Does stand-alone ship before collaboration? | Yes | Release 1 has its own acceptance criteria. |
| Existing records? | Start fresh. An XML import add-on, easy for an LLM to produce, may come later and is not specified. | No import in the early releases. Keep the data model import-friendly. |
| What output feeds invoicing? | Per client, per project, per month; separated by category; total hours plus the list of descriptions. An LLM summarizes and translates outside the app. | A **monthly summary** with copyable text is a first-release feature. |
| Are rates needed? | Rates are internal. The billed rate lives in the invoicing software. Rates matter only once a reporting module exists. | Rates, values and rate conflicts move to a later reporting phase. |
| Platforms? | Macs only | macOS only. Windows, Linux and phone are later. |
| Which entry method first? | Start and end, then live timer, then duration only. Baden uses start and end. Doug would use a timer if there were one. | The timer must exist before Doug joins. |
| Who runs the hub? | Baden, for three users | A hub on a server with a domain (architecture 5.9, option A). One-command setup is welcome, not essential. |
| Scope or date limits? | None stated; sequence logically | This document. |

---

# 2. Scope changes against Concept v2 (to confirm)

Concept v2 bundled stand-alone and one hub into its first milestone. These answers allow a smaller, earlier first release.

| Item | Concept v2 | Proposed |
|---|---|---|
| Rates, rate rules, rate conflicts, entry value, currency | First milestone | Later (release 5). Removes the rate-conflict alert and money rounding from early releases. |
| Invoice locks | First milestone | Release 3, on **hours only** (no values). |
| Owner amendments | First milestone | Release 4, only if the group needs them. |
| Roles | Collaborator, Project manager, Owner, with per-person overrides | Release 2 has Owner and Collaborator only. Project manager and overrides come in release 4. |
| Ownership transfer | First milestone | Release 4. |
| Removal of a member | Pending removal until contributions close | Release 2: removal is immediate (owner can always force it). Pending removal arrives with invoice closing in release 3, since "open contributions" has no meaning before then. |
| Automatic split on a time-zone change | Not in the concept | Release 4. Each entry records its own zone from release 1. |
| Monthly summary output | Reporting and export deferred | Pulled into release 1 in minimal form. |
| Import | Deferred | Stays deferred (release 5). |
| Languages beyond English, public API | Planned | Release 5. Message catalogs are built from the start. |

---

# 3. Sequencing principles

1. Every release is usable on its own and has acceptance criteria and a real-use period (one billing month).
2. Release 1's data model is **sync-ready from the first day**: client-generated time-ordered IDs, a revision on every record, author, device and origin metadata, tombstones, and an append-only history. Release 2 should not need a data migration of meaning.
3. The shared Rust core exists from the start. The concept's test vectors (§18) are test fixtures from the first commit.
4. The synchronization approach is decided by a spike (architecture §7) before the hub is built.
5. Nothing before release 2 depends on a hub.
6. Scenario walk-throughs (architecture 5.1 to 5.7) become executable tests.

---

# 4. Roadmap

| Phase | Delivers | Who gains | Depends on | Size |
|---|---|---|---|---|
| **P0** Foundations | Repository, core skeleton, fixtures, storage, IDs | Developers | Decisions D1 to D4 | S to M |
| **R1** Stand-alone | Full time tracking on one Mac, monthly summary | Baden | P0 | L |
| **S** Sync spike | Decision on the synchronization approach, scenario tests | Developers | A stable data model from R1 | M |
| **R2** One hub | Accounts, sync, projects with members, summary across members | Baden, Doug, Steffen | R1, S | XL |
| **R3** Invoice locks | Contributor and client invoice locks on hours | Baden, Doug, Steffen | R2 | M |
| **R4** Authority and roles | Project manager, overrides, ownership transfer, amendments, zone-change split | As needed | R3 | M to L |
| **R5** Reporting and more | Rates, reporting module, XML import, translations, API | As needed | R3 | Open |
| Later | Windows, Linux, phone, hosted hubs, federation | | | |

```
 P0 ──► R1 ─────────────────► R2 ──► R3 ──► R4
         └──► S (spike) ──────┘          └──► R5
```

---

# 5. Phases

## P0 Foundations

**Goal.** A repository where the core can be built and tested before any screen exists.

**Scope.**
- Repository layout, license, build and continuous testing.
- Rust workspace: core, desktop application, hub placeholder.
- Fixtures from the concept (§18): duration arithmetic, midnight splits, the DST vector (23:00 on 24 October to 04:00 on 25 October in Berlin: 1:00 + 5:00, real 6:00).
- Record model: time-ordered IDs, revisions, provenance, tombstones, history records.
- Storage behind an interface, implemented on SQLite.
- Message catalogs (English), so no text is hard-coded.

**Acceptance.**
- The fixtures pass in automated tests on every change.
- In a test harness with no screen, a record can be created, revised, deleted and reconstructed with its full history.
- Two independent test databases can exchange nothing yet, but their records never collide in ID.

**Decisions needed first.** D1 to D4 in section 7.

## R1 Stand-alone

**Goal.** Baden stops using the Google Doc.

**Scope.**
- First run with the stand-alone choice (architecture 5.9a).
- Clients, projects, categories. Archive; hard delete only in a danger zone.
- Time entries in this order of attention: start and end, live timer, duration only. Description required. Billable flag. Tags (free-form).
- Midnight split performed by the client, with each entry's zone and offsets recorded.
- Row-level editing in the list (Enter saves, Esc cancels). Day and week views over a 60-day default window; search over all entries.
- Hours as hh:mm. Preferences: date format, 24-hour clock, language (English).
- Stale-timer prompt as the only notification.
- Append-only change history.
- Backup and restore.
- **Monthly summary:** for a chosen month, per client and project, by category: total hours and the list of descriptions. Copy as plain text.

**Not included.** Hub, sync, rates and values, invoice locks, import.

**Acceptance.**
1. Baden records a full billing month in TimeTrack instead of the Google Doc, and the summary matches the manual figures.
2. The app works with the network disabled.
3. The concept's fixtures pass inside the real application, including the DST vector.
4. An entry can be created, edited and finished from the keyboard.
5. Backup then restore on a fresh installation reproduces identical data.
6. Quitting or crashing during a timer does not lose the timer; the stale-timer prompt appears.
7. Every edit is visible in the history with before and after.
8. The summary's category totals add up to the project total, and every description appears.

**Decisions needed first.** Summary text format (D5). Whether a menu-bar timer and a global shortcut are in release 1 (D6). Doug depends on a timer, so a menu-bar timer is recommended before he joins.

**Risks.** Tauri and macOS integration. The timing of time entry. Pulling hub concerns into the first release.

## S Sync spike

**Goal.** Decide how data moves between devices and the hub, on evidence.

**Scope.** As architecture §7.2: scenarios written as tests, a custom operation log on SQLite, a PowerSync prototype, optionally an Automerge or Loro prototype, then a comparison.

**Acceptance.**
- Scenarios 5.1, 5.2, 5.3 and 5.4 run against the leading approach and pass.
- The decision and the reasons are written down. The same scenarios become the conformance tests for release 2.

**Gate.** Release 2 does not start before this is decided.

## R2 One hub

**Goal.** Doug and Steffen record on their own Macs, and Baden sees every hour without asking.

**Scope.**
- Hub application: storage chosen in the spike, structured logs without descriptions, encrypted backups.
- Operator command line: create account, issue invite, disable or enable, backup, restore.
- Minimal sign-in pages with passkeys. TLS through a reverse proxy, with a setup guide.
- First run, join an existing hub (architecture 5.9b1). Device registration.
- Sync of one person's entries across their devices.
- Projects with members: Owner and Collaborator. Invitation links. The hub filters what each person receives (owner sees all entries, a collaborator sees their own).
- Core capability evaluator used by app and hub.
- Conflicts: field groups, side-by-side resolution, needs-attention list.
- Import of a stand-alone installation into a hub, with alias for a second install (architecture 5.6, 5.9).
- Immediate removal of a member.
- Sync status display.
- Monthly summary across all members of a project, with an optional breakdown by member.

**Not included.** Invoice locks, Project manager, overrides, ownership transfer, amendments, rates.

**Acceptance.**
1. Doug and Steffen record offline for a week, reconnect, and Baden's monthly summary lists all their hours and descriptions.
2. Scenarios 5.1, 5.2 and 5.6 pass as automated tests.
3. Interrupting the network repeatedly during sync leaves no duplicates and loses nothing.
4. A hub restored from backup is identical in content, and clients resume without duplicates.
5. A removed member receives no new data and their queued work is kept as drafts.
6. Hub logs contain no descriptions.
7. All three use it for one billing month.

**Decisions needed first.** The spike outcome. Hub packaging and storage. TLS approach. Passkey page approach (D7).

**Risks.** Synchronization (the largest). Passkey ceremonies in a small hub page. Protocol versioning.

## R3 Invoice locks

**Goal.** Nothing already billed changes afterwards. This addresses Doug's October-then-November adjustment problem.

**Scope.**
- Client invoice close by the owner and contributor invoice close by the member, both on hours only. Independent locks. Optional external reference.
- Late entries stay uninvoiced and go on the next invoice.
- Reopen with a mandatory reason, with the cross-lock rule.
- Entries with an open conflict: excluded from a client invoice and listed; a contributor invoice is refused until resolved.
- Owner's close dialog shows each member's status and last sync.
- Pending removal until a member's contributions are closed. Forced removal by the owner.
- Refusals shown as drafts with the reason.

**Acceptance.**
- Scenario 5.4 passes, including two closes racing and an offline edit arriving after the close.
- A locked entry cannot be changed by anyone.
- The October scenario works end to end across three users.
- Summary distinguishes invoiced from uninvoiced hours.

## R4 Authority and roles

Built when the group needs it, or when a fourth person joins.

**Scope.** Project manager preset and per-person overrides. Ownership transfer with the former owner's chosen status (architecture 5.3). Authority epochs in refusals. Role change while offline (5.5). Owner amendments, if wanted. Automatic split on a time-zone change (architecture 5.7). Pending transfer expiry.

**Acceptance.** Scenarios 5.3, 5.5 and 5.7 pass.

## R5 Reporting, rates and extras

Unordered. Candidates: rates and values (concept §10), a reporting module, CSV export, the XML import add-on, additional UI languages (German first), the public API.

## Later

Windows, Linux and phone clients. Hosted hubs. Independent hubs and federation (concept phases 3 and 4).

---

# 6. Cross-cutting work

| Area | What |
|---|---|
| Testing | Fixtures from the concept; the walk-throughs as scenario tests; property tests for splits; a shared validator run by client and hub. |
| Security | TLS, access control, encrypted backups, logs without descriptions or rates, protocol versioning. |
| Localization | Message catalogs from P0. |
| Operations | Operator guide, backup and restore procedure, upgrade procedure. |
| Migration | Schema versions in the app; protocol versions between app and hub. |

---

# 7. Decisions and open items

| # | Decision | Needed by | Recommendation |
|---|---|---|---|
| D1 | Confirm Tauri with a TypeScript interface | P0 | Yes |
| D2 | Open-source license (check against any sync engine's license) | P0 | Decide with the repository |
| D3 | Repository layout (one workspace) | P0 | One workspace |
| D4 | ID and clock scheme for time-ordered, never-reused IDs | P0 | Decide in P0 with a short note |
| D5 | Monthly summary format | R1 | Plain text grouped by client, project, category; descriptions as lines |
| D6 | Menu-bar timer and global shortcut in R1 | R1 | Yes, because Doug will use a timer |
| D7 | Passkey page approach and library | R2 | Decide after the spike |
| D8 | Who builds and how many hours per week | Plan to calendar | Needed from you |
| D9 | Hub packaging, storage and TLS | R2 | Architecture 5.9, questions 3 and 4 |
| D10 | Scope changes in section 2 | Now | **Confirmed** (3 November 2026) |

Decided 3 November 2026: D1 (Tauri), D2 (Apache-2.0 or MIT), D3, D4, D5, D6 (menu-bar timer in R1). D8: Baden and Claude build; Doug and Steffen test. Task breakdown for P0 and R1: [dev-plan-p0-r1.md](dev-plan-p0-r1.md).

Concept open items still apply (README §22), especially #4 (password method) and #26 (rate for a staying former owner, which moves to R5 with rates).

---

# 8. Risks

| Risk | Effect | Response |
|---|---|---|
| Synchronization is harder than expected | Release 2 slips | The spike comes first. Scenario tests are written before the hub. |
| Sync-ready data model is skipped in R1 | Migration pain in R2 | P0 sets the record model; R1 acceptance includes history and revisions. |
| Passkey page proves heavy for a small hub | Sign-in blocks release 2 | Prototype in the spike. Keep the page minimal. |
| Scope creep in R1 from hub concerns | R1 delayed | Hub items listed as not included. |
| Time-zone edge cases | Wrong dates | Recorded offsets and shared fixtures; zone-change split deferred. |
| Single operator | Outages, lost backups | Documented backup and restore, tested in R2 acceptance. |

---

# 9. Next steps

1. Confirm the scope changes in section 2 (D10).
2. Decide D1 to D4 and start P0.
3. Answer D8 so the sizes can become a calendar.
4. Continue the create-a-hub deep dive (architecture 5.9) in time for release 2.
