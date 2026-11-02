# TimeTrack — Architecture Direction

**Status:** Draft. Working notes, not an approved design.
**Concept:** [README.md](../README.md) (Concept v2) states what the product does and stays technology-agnostic. This document holds the architecture ideas that follow from it.
**Sources:** [timetrack_architecture_decisions_2026-11-02.md](timetrack_architecture_decisions_2026-11-02.md) and [timetrack_architecture_reconstruction_2026-11-02_1400.md](timetrack_architecture_reconstruction_2026-11-02_1400.md). The second is a reconstruction of a conversation that could not be recovered verbatim, so the items it reports as later decisions are recorded here as decisions on your instruction, without independent verification.

---

# 1. Decisions so far

| Area | Decision |
|---|---|
| Shared business logic | A **shared Rust domain core** holds the business rules and data-model invariants. |
| Topology | **Separate applications**: a desktop client and a hub. Both use the same core. They do not reimplement the rules. |
| Standalone | The core works without a running hub. |
| Delivery | macOS desktop first, plus one optional sync hub. |
| Interface and shell | Preference: Tauri with a TypeScript interface over the Rust core. Not final. |
| Local storage | SQLite is the leading candidate. |
| Sync | Open. Separate synchronization from authority (§6); choose after the spike (§7). |
| Everything else | Open (§8). |

```
 ┌───────────────────────┐             ┌───────────────────────┐
 │   Desktop application │             │   Hub application      │
 │   UI, OS integration  │   sync      │   network API, sign-in │
 │   local storage       │◄──────────►│   pages, storage        │
 ├───────────────────────┤  protocol   ├───────────────────────┤
 │      Shared Rust domain core        │ (same core, both sides) │
 └───────────────────────┴─────────────┴───────────────────────┘
```

---

# 2. What the core owns

The core is the single implementation of the rules in the concept. It contains:

- identifiers, versions and the metadata every record carries (author, device, origin, provenance)
- time calculation: durations, midnight splitting and merging, validation, timezone handling. Splitting runs in the author's client; the hub runs only the consistency checks on the recorded result
- rate resolution, effective dating, conflict detection, entry value
- the capability evaluator (§4 of the concept)
- entry versions, owner amendments and the effective figure
- invoice closing, snapshots and lock rules
- conflict detection and the data needed to present and resolve one
- construction and validation of change-history records
- validation of incoming and outgoing sync changes

It does not contain: the user interface, operating-system integration, network transport, the hub's web pages and passkey ceremonies, or file dialogs.

---

# 3. Design implications

These follow from the goals. They are recommendations, not recorded decisions.

1. **Isolate the domain** from the interface and the network layers. The core has no knowledge of either.
2. **Record identity explicitly:** object identity, writer identity, provenance and version on every record, so offline replication works.
3. **One authorization decision everywhere.** A single evaluator answers for interface actions, local changes, queued sync changes, hub routing and a future API.
4. **Conflict resolution is a domain operation.** It references both competing revisions and writes to the history. It is never an invisible overwrite.
5. **Separate source facts from interpretation.** Contributor entries are immutable source facts. Owner amendments are append-only interpretations layered on top.
6. **Deterministic arithmetic.** The same inputs must give the same outputs on the desktop and on the hub. The clock is an input, not something the core reads, which also makes tests simple.
7. **Record time-zone interpretation, do not recompute it.** Which calendar day an entry belongs to, and where a midnight split falls, depend on timezone rules that change over time and differ between installations. The author's client computes them once, when the time is recorded, and stores the result with its context: instants in UTC, the IANA zone, the offsets in force, and the database version. The hub checks structure from those recorded values and never recalculates. Neither side needs the other's timezone data, so no version has to be pinned. Recalculation happens only when the author edits an entry's timing.
8. **Storage behind an interface.** The core defines what it needs from storage. The desktop uses a local embedded database. Whether the hub shares that implementation, or uses a different database behind the same interface, is open.

---

# 4. Consequences for the earlier stack discussion

- The earlier suggestion of a shared **TypeScript** core is superseded by the Rust core.
- **Tauri** (a Rust shell around a web interface) now fits naturally: the core is already in the process. It is not yet decided.
- Other interface options become realistic because the core is language-neutral at its edge: a native **SwiftUI** interface for macOS through bindings, or a Rust-native interface framework. This also keeps a later web client possible by compiling the core to WebAssembly.
- **Electron** loses most of its advantage, since sharing code with a JavaScript main process would no longer be the point.
- **Sync engines** are not ruled out. The working split is: a synchronization layer moves, retries and deduplicates changes; the Rust core decides which changes are valid (§6). Candidates and the plan to compare them are in §7.

---

# 5. Scenarios that will drive the protocol

Before choosing a replication design, walk through these with the concept's rules:

1. **Concurrent edit** of one time entry on two devices of the same person, with overlapping and non-overlapping fields.
2. **Revoked collaborator** whose offline device still holds queued entries.
3. **Ownership transfer** while one device is offline and holds queued edits.
4. **Invoice close** racing an offline edit.
5. **Role change** while a device is offline.
6. **Joining a hub** from a standalone installation with existing local projects.
7. **Timezone data drift:** a client with newer timezone rules than the hub, and a person recording time while travelling in another zone.

Each scenario states the expected result, then shows what the protocol must do to produce it.

---

# 6. Synchronization versus authority

**Direction (agreed):** reuse an existing synchronization mechanism for moving and reconciling data if one fits, and keep authority in the Rust core. A synchronization layer does not need to know what an owner is. It transmits changes reliably; the core on the hub decides which are valid.

```
 Desktop app: local SQLite + Rust core
        │  queued operations ▲ accepted operations
        ▼                    │
 Synchronization layer: discover · transfer · retry · deduplicate · track versions
        │                    ▲
        ▼                    │
 Hub: Rust core validates authority, enforces permissions, records accepted changes, distributes
```

## 6.1 Three kinds of change

| Kind | Meaning | Examples |
|---|---|---|
| Mergeable | Independent changes with no clash | Two people recording entries; one person's entries from two devices |
| Conflicting but valid | Both changes are legitimate; neither may be silently lost | Two different edits to one entry's description. Preserved and shown side by side (concept §13.3) |
| Authority-sensitive | Validity depends on who holds authority at that moment | Ownership transfer, membership and role changes, rate rules, amendments, invoice close, project delete |

The synchronization layer must carry all three without confusing them. A convergent merge (as in a CRDT) establishes agreement on values, not whether an operation was authorized.

## 6.2 Provisional or authoritative offline (recommendation, not yet decided)

**Optimistic locally, provisional until the hub accepts.**

- **Author-owned changes** (own entries, personal tags, own invoices, preferences) take effect on the device at once and are queued. The hub accepts or rejects them on receipt. Authors are the only writers of their records, so concurrent edits come only from one person's own devices.
- **Authority-sensitive changes need the hub.** Offline they are queued requests, not actions. The person is told the change is pending. This removes the stale-authority race: an offline device cannot exercise authority that has since moved.
- **Standalone installation:** everything is final at once, since there is no hub. On joining a hub, existing data becomes proposals that the hub validates (scenario 6).
- **Validation uses the authorization state at receipt**, not at creation. Device clocks cannot be trusted to establish what was permitted when. Stricter than accepting pre-revocation work, but unambiguous.
- **Rejection is never silent.** A rejected operation stays on the device as a visible draft with the reason, so work is not lost (concept §13: no silent loss).

Open choice: accept work made before a revocation if its creation time could be trusted. Not recommended while creation time comes from the device.

## 6.3 Consequence: operations with a hub-assigned order

With one hub as sequencer (Phases 1 and 2), the natural model is an **operation log**: each change records intent, not just resulting rows.

```
{ operation_id, actor_id, device_id, type: "time_entry.update",
  entry_id, base_revision, changes: { description: "..." } }
```

Illustrative, not a schema. The hub assigns each accepted operation a sequence number. Clients keep a queue of unaccepted operations and a cursor into the accepted log. This gives explicit conflict detection from `base_revision`, idempotent replay through `operation_id`, and an audit trail as a by-product. Still to design: snapshots for new installs, authorization epochs, rejected-operation handling, safe log compaction, protocol versioning.

Peer-to-peer merging without a sequencer is only needed from Phase 3 and 4 (independent hubs, federation).

---

# 7. Synchronization candidates and spike plan

## 7.1 Candidates

Findings marked **verified** come from a web search on 2026-11-02 of vendor documentation and package pages. Everything else is from memory and still unconfirmed.

| Candidate | Fit | Concerns |
|---|---|---|
| **Custom operation log on SQLite** | Exact control; matches §6.3 | We build transport, replay, snapshots, compaction and compatibility ourselves |
| **PowerSync** | Same shape: client mutation queue, backend validates uploads, per-user sync rules | See below |
| **Automerge** | Mature offline collaboration; Rust implementation; sync state tracks what each peer lacks | Automatic merge does not enforce our rules and conflicts with side-by-side resolution. Revisit for Phase 4 |
| **Loro** | Rust CRDT; version-vector incremental updates | Same as Automerge. Its README says the API and encoding are experimental (verified) |
| **SQLite session extension** | Native changesets and conflict callbacks | Database conflicts are not business conflicts. Possible capture mechanism only |
| **ElectricSQL** | Postgres read-path sync | Writes handled separately; lowest priority |

**PowerSync, verified:**
- A Tauri SDK exists (`tauri-plugin-powersync`), built on the Rust SDK. Both are described as **alpha**; the standalone `powersync` crate as pre-alpha with no stability guarantee. Connecting to the service is possible only from Rust.
- The self-hosted service needs **Postgres** (logical replication) as the source **and MongoDB** for sync-bucket state. Production guidance recommends a three-node MongoDB replica set. With our hub that is at least three services, a heavy load for a small operator.
- The Open Edition is under the Functional Source License (converts to an open license after two years), is labelled Beta, and the client SDKs are Apache-2.0. Whether FSL is acceptable for our own open-source license is open item 10.
- Not verified: how upload (write-back) validation and conflict handling work. Needs a documentation read.

**Automerge and Loro, verified:** the search confirmed per-peer sync state and version-vector incremental updates. It found nothing on authorization or access control, consistent with the concern that this is left to the application.

**Assessment:** the stated server requirements make PowerSync a poor match for "one small hub a person can run", and its Rust and Tauri SDKs are alpha. It stays in the comparison to test its model, not as a likely choice.

## 7.2 Spike plan

1. **Write scenarios.** Turn scenarios 1 to 7 in §5 into written test cases with expected results. Core five: offline recording without duplicates, concurrent edit with both versions surfaced, ownership transfer with chosen former-owner role, revocation with queued offline work, new install rebuilding correct state.
2. **Prototype A, custom operation log:** Rust core, SQLite on client and hub, hub as a second process, sequence numbers, a queue on the client.
3. **Prototype B, PowerSync:** same scenarios, judged on services required, how rejections and side-by-side conflicts are expressed, and effort.
4. **Optional C, Automerge or Loro,** only for the documents where convergence helps, to confirm the earlier assessment.
5. **Compare** on: scenarios passed, operational weight for a small operator, license fit, maturity, how much of the core's authority logic leaks into the engine.

Decision after the spike, not before.

---

# 8. Open questions

| # | Question |
|---|---|
| 1 | Interface framework and how it talks to the core (bindings). Current preference: Tauri with a TypeScript interface |
| 2 | Operating-system integration: tray or menu bar, global shortcut, auto-start, secure storage |
| 3 | Whether the hub reuses the desktop's persistence layer, and the hub's database |
| 4 | API transport between desktop and hub |
| 5 | Replication model: operation log with hub-assigned order (§6.3) versus an existing engine. Decided by the spike (§7.2) |
| 5a | Offline authority: confirm the provisional-until-accepted split and receipt-time validation (§6.2) |
| 6 | Identifier scheme and ordering of one person's edits across devices |
| 7 | How the hub serves its minimal sign-in pages |
| 8 | Deployment tooling for the hub |
| 9 | Test strategy for the core, including shared test vectors from the concept |
| 10 | Open-source license and repository layout |
