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
| Everything else | Open (§6). |

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
- **Sync engines** that assume one central database (ElectricSQL, PowerSync and similar) fit poorly with core-owned authority and explicit conflicts. The sync protocol is likely to be small, custom and specified from §13 of the concept. This should be checked against the current state of those tools before deciding.

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

# 6. Open questions

| # | Question |
|---|---|
| 1 | Interface framework and how it talks to the core (bindings) |
| 2 | Operating-system integration: tray or menu bar, global shortcut, auto-start, secure storage |
| 3 | Whether the hub reuses the desktop's persistence layer, and the hub's database |
| 4 | API transport between desktop and hub |
| 5 | Replication model: event log, versioned state, or another approach |
| 6 | Identifier scheme and ordering of one person's edits across devices |
| 7 | How the hub serves its minimal sign-in pages |
| 8 | Deployment tooling for the hub |
| 9 | Test strategy for the core, including shared test vectors from the concept |
| 10 | Open-source license and repository layout |
