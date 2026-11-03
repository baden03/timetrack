# TimeTrack — Phased Rollout Plan

**Status:** Working plan, updated 3 November 2026. **Release 1 (stand-alone) is built and its real-use month has started** (version 0.0.3, a pre-release on GitHub). From 3 November to about 3 December 2026 Baden records all his time in TimeTrack instead of the Google Doc, and Doug and Steffen use the same build from the quick-start guide. Feedback is collected as the month unfolds. The sync spike and release 2 have not started. Section 10 records what has been delivered.
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

# 2. Scope changes against Concept v2 (confirmed 3 November 2026)

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
| **P0** Foundations | Repository, core skeleton, fixtures, storage, IDs | Developers | Decisions D1 to D4 | S to M | **Done** |
| **R1** Stand-alone | Full time tracking on one Mac, monthly summary | Baden | P0 | L | **Built; real-use month running** (v0.0.3, started 3 Nov 2026, ends about 3 Dec) |
| **S** Sync spike | Decision on the synchronization approach, scenario tests | Developers | A stable data model from R1 | M | Not started |
| **R2** One hub | Accounts, sync, projects with members, summary across members | Baden, Doug, Steffen | R1, S | XL | Not started |
| **R3** Invoice locks | Contributor and client invoice locks on hours | Baden, Doug, Steffen | R2 | M | Not started |
| **R4** Authority and roles | Project manager, overrides, ownership transfer, amendments, zone-change split | As needed | R3 | M to L | Not started |
| **R5** Reporting and more | Rates, reporting module, XML import, more translations, API, update check | As needed | R3 | Open | German done early (see R1); the rest not started |
| Later | Windows, Linux, phone, hosted hubs, federation | | | | |

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
- Hours as hh:mm. Preferences: date format, 24-hour clock, language (English and German), theme, backup reminder.
- Stale-timer prompt as the only notification.
- Append-only change history.
- Backup and restore.
- **Monthly summary:** for a chosen month, per client and project, by category: total hours and the list of descriptions. Copy as plain text.

**Not included.** Hub, sync, rates and values, invoice locks, import.

**Status (3 November 2026).** All the scope above is built, plus additions that came out of testing (listed in section 10). The eight acceptance criteria stand as follows; the evidence for each is in [dev-plan-p0-r1.md](dev-plan-p0-r1.md), section 9.
- **Met by tests:** 3 (fixtures and daylight saving), 6 (timer survives a crash), 7 (history), 8 (summary totals), and the data side of 5 (backup then restore reproduces records and history exactly).
- **Needs a person at a real Mac:** 1 (Baden's real billing month), 2 (running with Wi-Fi off; the code and checks already rule out network use), 4 (keyboard use, verified in a browser; repeat in the built app) and the second-Mac half of 5.
- **Also not yet seen on a real desktop:** the menu-bar text, the Cmd+Shift+T shortcut, the tray menu, the macOS About panel and the native file dialogs for backup and restore.

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

**The spike in plain terms.**

*The problem.* In release 2 each person has a copy of their data on their own Mac, and a hub (a small server) holds the shared copy. People work offline and reconnect later, so two copies of the same entry can change independently. The software that moves changes around, notices when two changes clash, and ends with every copy agreeing is called a **sync engine**. Getting this wrong loses hours or duplicates them, which in a billing tool is the worst failure.

*Why a spike.* A spike is a short, throwaway experiment that answers one question so we do not guess. Here the question is: which way of building sync fits TimeTrack? Changing the answer after the hub is built would be very expensive, so we pay a small price now. Spike code is not shipped; the output is a decision and a document.

*Step 1: write the scenarios as tests first.* Architecture §5 describes situations in words, for example: Doug and Baden edit the same entry offline (5.1); a removed collaborator reconnects with queued work (5.2); ownership moves while a device is offline (5.3); an invoice closes while someone's offline edit is still on its way (5.4). We turn each into an automated test with an exact expected outcome. These tests are the yardstick, and they are reused later as the acceptance tests for release 2.

*Step 2: try the candidates against the same tests.*
- **Custom operation log (Prototype A).** Every change is recorded as a small numbered operation ("set the end time of entry X"). Devices send their operations to the hub; the hub puts them in one order and sends back what each device is missing. We write all of this ourselves. It gives full control and matches our rules, but it is more code for us to own.
- **PowerSync (Prototype B).** A ready-made engine with the same overall shape. We test whether it can express our rules (refusing a change, showing both versions side by side) and what it needs to run. Architecture §7.1 already records worries: its self-hosted server needs Postgres and MongoDB, and its Tauri and Rust pieces are alpha. It is in the comparison to test its model, not as the likely winner.
- **Automerge or Loro (optional Prototype C).** Libraries that merge concurrent edits automatically (CRDTs). Automatic merging is the opposite of what we want for clashes, because we show both versions and let a person choose, and they do not enforce who may do what. Only worth building if A and B both disappoint.

*Step 3: compare and decide.* For each candidate record: which scenarios passed, how heavy it is for one person to run as a hub, license fit, maturity, and how much of our permission logic ends up inside the engine. Write the recommendation and reasons in `docs/`, and note whether the R1 data model needs any change.

*Rules for the spike.* It reads the existing R1 record model (IDs, revisions, authors, tombstones, history) and does not change the R1 app. If it shows the model needs a change, that goes in the write-up and is handled after the test month, so Baden's real data is never put at risk.

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

Unordered. Candidates: rates and values (concept §10), a reporting module, CSV export, the XML import add-on, more UI languages (German is done and [translating.md](translating.md) explains how to add others; languages that need more than two plural forms need a small extension), the public API.

**Check for updates** (requested 3 November 2026, on the About screen). Release 1 makes no network connection at all, and `scripts/no-network.sh` enforces that, so this needs a deliberate decision when it is built:
- It must be an opt-in button (concept §16.1), never a background check, and the only connection the app makes.
- Updates must be signed (Tauri's updater uses its own signing key, kept off the repository) and hosted somewhere stable, for example GitHub Releases.
- macOS will only let a downloaded update run smoothly if the app is signed with an Apple developer certificate and notarized; the current ad-hoc signature is not enough.
- The guard script and the content security policy get one explicit, documented exception for the update address.

## Later

Windows, Linux and phone clients. Hosted hubs. Independent hubs and federation (concept phases 3 and 4).

---

# 6. Cross-cutting work

| Area | What |
|---|---|
| Testing | Fixtures from the concept; the walk-throughs as scenario tests; property tests for splits; a shared validator run by client and hub. |
| Security | TLS, access control, encrypted backups, logs without descriptions or rates, protocol versioning. |
| Localization | Message catalogs from P0. English and German are complete, with a test that keeps every language in step. |
| Operations | Operator guide, backup and restore procedure, upgrade procedure. |
| Migration | Schema versions in the app; protocol versions between app and hub. |

---

# 7. Decisions and open items

| # | Decision | Needed by | Recommendation |
|---|---|---|---|
| D1 | Confirm Tauri with a TypeScript interface | P0 | **Decided: yes.** React UI over the Rust core; the menu-bar timer and shortcut worked in the P0 spike |
| D2 | Open-source license (check against any sync engine's license) | P0 | **Decided: Apache-2.0.** Re-check when a sync engine is chosen |
| D3 | Repository layout (one workspace) | P0 | **Decided:** Rust workspace (`core`, `storage`, `app`, `hub` placeholder) plus the desktop app |
| D4 | ID and clock scheme for time-ordered, never-reused IDs | P0 | **Decided:** UUID v7 with an origin suffix; see [ids-and-clocks.md](ids-and-clocks.md) |
| D5 | Monthly summary format | R1 | **Decided:** see [summary-format.md](summary-format.md) (billable only; bullet descriptions are one line each) |
| D6 | Menu-bar timer and global shortcut in R1 | R1 | **Decided: yes, built** |
| D7 | Passkey page approach and library | R2 | Open; decide after the spike |
| D8 | Who builds and how many hours per week | Plan to calendar | **Decided:** Baden and Claude build; Doug and Steffen test. Hours per week still not stated, so sizes remain relative |
| D9 | Hub packaging, storage and TLS | R2 | Open; architecture 5.9, questions 3 and 4 |
| D10 | Scope changes in section 2 | Now | **Confirmed** (3 November 2026) |
| D11 | Check for updates in the app | After R1 | Open. Needs an opt-in button, signed updates and an Apple developer certificate with notarization; see the R5 note |
| D12 | Apple developer signing and notarization, and an Intel or universal build | Before wider testing | Open. Builds are ad-hoc signed for Apple Silicon only; testers use right-click, Open once |
| D13 | Continuous integration | Done | **Decided:** runs by hand and on a version tag, not on every push; the macOS desktop job is manual because GitHub's Apple Silicon runners are often at capacity. `scripts/check.sh` runs the same checks locally |

Task breakdown for P0 and R1, with the acceptance evidence: [dev-plan-p0-r1.md](dev-plan-p0-r1.md).

Concept open items still apply (README §22), especially #4 (password method) and #26 (rate for a staying former owner, which moves to R5 with rates).

---

# 8. Risks

| Risk | Effect | Response |
|---|---|---|
| Synchronization is harder than expected | Release 2 slips | The spike comes first. Scenario tests are written before the hub. |
| Sync-ready data model is skipped in R1 | Migration pain in R2 | P0 sets the record model; R1 acceptance includes history and revisions. |
| Passkey page proves heavy for a small hub | Sign-in blocks release 2 | Prototype in the spike. Keep the page minimal. |
| GitHub macOS runners are often at capacity | Remote checks wait or fail | Checks run locally (`scripts/check.sh`) before a release; the desktop CI job is manual (D13). |
| Builds are not notarized and are Apple Silicon only | Testers need a Gatekeeper workaround; Intel Macs cannot run it | Install note; decide D12 before wider testing. |
| Scope creep in R1 from hub concerns | R1 delayed | Hub items listed as not included. |
| Time-zone edge cases | Wrong dates | Recorded offsets and shared fixtures; zone-change split deferred. |
| Single operator | Outages, lost backups | Documented backup and restore, tested in R2 acceptance. |

---

# 9. Next steps

0. **Now (from 3 November 2026): the real-use month.** Baden uses 0.0.3 in place of the Google Doc and collects feedback as the month unfolds. Doug and Steffen install from the link and the quick-start guide and report as they go. During the month, only bugs and blockers are fixed, in numbered builds, so the summary can be compared with the manual figures without a moving target. At the end of the month the eight R1 acceptance items are marked met or open, and the next round of work is chosen from the collected feedback.
1. **Finish release 1.** Baden records a real billing month in TimeTrack and compares the summary with the manual figures. Check by hand on a real Mac the items listed under R1 (menu-bar text, shortcut, tray menu, About panel, backup and restore dialogs, Wi-Fi off, a restore on a second Mac). Doug and Steffen try the build and report on the timer and the flow.
2. **Collect and act on feedback** as short rounds, as done so far; each round ends in a numbered test build.
3. **Start the sync spike (S)** while the real-use month runs: write the architecture scenarios as tests, build the custom operation-log prototype and compare it with the alternatives (architecture §7.2). Decide the sync approach.
4. **Decide the open items for release 2:** D7 (passkey page), D9 (hub packaging, storage, TLS), and, before wider testing, D12 (signing, notarization, Intel).
5. **Break release 2 into tasks** the way P0 and R1 were, once the spike has decided.
6. **Later in release 1's life:** decide D11 (the update check) and write it as a deliberate exception to the no-network rule.

---

# 10. Delivered so far

Releases are pre-releases on GitHub, built on Apple Silicon Macs and signed ad hoc.

| Version | Date | What it added |
|---|---|---|
| 0.0.1 | 3 Nov 2026 | The first complete stand-alone build: clients, projects, categories (optional on an entry); a dashboard of project cards and a project page; entries by start and end, timer or duration; midnight split with time zones; menu-bar timer and Cmd+Shift+T, kept in step with the window; week view, search, history; monthly summary; backup and restore with a reminder; date and time formats and checks on leaving a field; light and dark themes; keyboard use; no network use. |
| 0.0.2 | 3 Nov 2026 | German interface (informal), a language setting, the About screen, an icon with transparent corners. |
| 0.0.3 | 3 Nov 2026 | Notes added while a timer runs (bullet points of the entry); descriptions typed a line at a time; category defaults to the last used; colors for clients and projects; end times up to 15 minutes ahead; duration shorthand (`:45`, `.75`, `1h30`); tidier lists (no Tags column, tags after the description, "Time" header) and form controls of one height. |

**Added beyond the original R1 scope, from testing feedback:** the dashboard and project page, an optional category, running notes with bullet descriptions, colors, the 15-minute allowance, duration and date and time shorthand with checks on leaving a field, German, the About screen, the CI change, and a quick-start guide.

**Documents:** [quick-start.md](quick-start.md) (setting up a new install), [install-macos.md](install-macos.md) (installing the build), [translating.md](translating.md) (adding a language), [summary-format.md](summary-format.md), [ids-and-clocks.md](ids-and-clocks.md), [spike-tauri.md](spike-tauri.md).
