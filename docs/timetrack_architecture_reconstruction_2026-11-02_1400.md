# TimeTrack — Architecture Workshop Reconstruction

**Date:** November 2, 2026, approximately 2:00–2:10 p.m. (Costa Rica, UTC−6)  
**Project:** https://github.com/baden03/timetrack  
**Document status:** Reconstructed working notes, **not a recovered verbatim transcript**.

## Provenance and confidence

This document reconstructs the **later architecture conversation** described in the surviving conversation excerpts. The original **inline Markdown block** has not been located. It is distinct from the earlier file `timetrack_architecture_decisions_2026-11-02.md`.

Labels used below:
- **Earlier established:** explicitly present in the recoverable earlier architecture-decisions file.
- **Reported later decision:** described in the subsequent conversation recap, but the underlying detailed conversation and original Markdown block are not available to verify independently.
- **Open/detail not established:** the sources do not establish the exact rule or its implementation.

## 1. Overall architecture

### Reported later decision — shared Rust core

- Build a **shared Rust domain core** for business rules and data-model invariants.
- Use that core in **separate desktop-client and hub applications**, rather than independently reimplementing the same rules.
- The core should be usable without a running hub, consistent with the local-first product vision.

**Not yet established:** desktop UI framework, operating-system integration, Rust-to-UI bindings, whether hubs reuse the exact persistence layer, API transport, and deployment tooling. Rust was **not** selected in the older architecture-decisions file; it appears in the later discussion recap.

### Earlier established — distributed product direction

- The desktop application works locally and offline, with no account or hub needed for basic tracking.
- A hub is optional for multi-device synchronization and collaborative projects.
- Hubs can be independently operated; no global master hub is required.
- Federation between independent hubs is a later phase, governed by explicit, directional administrator-controlled trust.

## 2. Project ownership and authority

### Earlier established

- A project initially has a single individual owner.
- Any installation/user can create projects, subject to applicable sharing and federation constraints.
- Collaborators retain authority over the time entries they create; the project owner cannot silently rewrite those source entries.
- Only the project owner defines the project's rates.
- Owner-side adjustments are separate records, preserving the contributor's original entry and provenance.

### Reported later decisions

- **Exactly one owner exists per project at any given time.**
- **Ownership transfer must be atomic**, so a transfer cannot leave a project ownerless or with two simultaneously authoritative owners.
- A project owner may adjust the owner's effective project accounting/reporting view without modifying a contributor's original time record.
- **Owner-only operations** include changing rate definitions, deleting the project, and transferring ownership; delegated managers do not obtain these powers merely by being managers.

**Still to specify:** ownership-transfer acceptance protocol, how offline peers learn of a transfer, what happens to pre-transfer queued edits, the semantics of deletion, and whether the owner-only operations can ever be explicitly delegated. The last point should not be assumed from a role label.

## 3. Permissions model

### Earlier established

Permissions are **project-scoped capabilities**, with convenient role presets and **per-user overrides**. They are not a rigid three-role hierarchy.

Baseline role concepts:

| Preset | Baseline access | Important limit |
|---|---|---|
| Collaborator | Access own contributed time and authorized project metadata | Does not automatically see other collaborators' entries |
| Project manager | Can be delegated visibility of all project entries and management of invitations/collaborators | Does not thereby gain owner-only authority |
| Owner | Full project visibility and ultimate project-level authority | Original contributor entries still retain their provenance |

### Reported later refinement

- Delegated **project managers** may manage collaborators, invitations, and permitted role assignments within the capabilities delegated to them.
- Management authority must not accidentally imply the ability to transfer project ownership, delete a project, or alter owner-controlled rates.
- Authorizations must be evaluated consistently for UI operations, local changes, queued synchronization events, hub routing, and future APIs.
- Permissions affect what is delivered during synchronization, not just what is hidden by the UI.

**Candidate capability groups for discussion, not a final approved capability matrix:**

| Capability category | Example operations |
|---|---|
| Read project | View permitted project metadata |
| Contribute time | Create or edit one's own source entries |
| View contributor time | Read others' time entries, if explicitly authorized |
| Manage collaborators | Invite/remove collaborators within delegated scope |
| Assign roles/capabilities | Change permitted access grants, without privilege escalation |
| Project accounting | View owner adjustments and applicable reports, if granted |
| Owner-only control | Define rates, delete project, transfer ownership |

**Still to specify:** exact grants, override precedence, default-deny behavior, role-change propagation to offline devices, revocation of access to data already cached, and whether managers can grant others the manager role. These are design questions, not settled mechanics.

## 4. Contributor entries vs. owner amendments

### Earlier established and reinforced later

- **Contributor source entries remain attributable to and controlled by the contributor.**
- A project owner who disputes or wants to reinterpret an entry creates a **separate amendment or adjustment**, preserving the source entry.
- That adjustment affects the **owner's reporting/accounting view immediately**, without overwriting the contributor's data or quietly propagating a modified source entry to them.
- The system needs to distinguish the **original assertion** from the **owner's effective accounting interpretation** and maintain a traceable link between the two.

**Still to specify:** adjustment permissions and visibility, approval or notification, effective date, audit format, export behavior, and cross-hub replication of amendments.

## 5. Synchronization and conflict resolution

### Earlier established

- Users can record time without an internet connection.
- Sync is asynchronous; hubs retain/deliver authorized changes to offline collaborators as they reconnect.
- Multiple devices per person are supported.
- A user's view is filtered according to project permissions across devices and hubs.

### Reported later decision — manual Git-style conflict handling

- Where genuinely competing edits cannot be reconciled safely, **identify and present the conflict explicitly**.
- Let the relevant user **choose which version to accept** instead of silently using last-write-wins.
- Preserve enough information to show the competing versions and their origins.

This is a **product-level conflict-resolution preference**, not yet a complete replication algorithm. The sources do **not** establish precisely which fields trigger conflicts, who is authorized to resolve a conflict, how conflicts are routed across hubs, or whether non-overlapping changes may merge automatically.

### Critical unresolved implementation questions

1. Which replica is authoritative for each class of object (project metadata, permissions, original time records, owner adjustments)?
2. How are stable identifiers and revisions generated while offline?
3. What constitutes concurrent modification, and how are versions compared?
4. Are event logs, revision vectors, CRDTs, or another protocol used internally?
5. What happens if a permission is revoked before a queued offline mutation arrives?
6. How are ownership transfers ordered against queued edits and federation events?
7. How is the audit trail preserved when a human resolves a conflict?

## 6. Hub federation

### Earlier established

- Independent open-source hubs, without a global authority.
- Hub-to-hub trust is **explicit and directional**; A trusting B does not automatically mean B trusts A.
- Administrators define federation policy; within allowed relationships, users should be able to collaborate without per-action administrator approval.
- Hub-level trust never bypasses project invitation and capability checks.

**Not settled:** identities and discovery across hubs, key management, authorization of cross-hub events, revocation, and conflict handling in partitions.

## 7. Development sequence

**Earlier agreed phases:**

1. **Standalone client:** local, offline time tracking and project ownership.
2. **Single hub:** optional multi-device sync and shared-project collaboration.
3. **Independent hubs:** multiple separately operated servers.
4. **Federation:** authorized collaboration between trusted hubs.

A later product recap additionally mentioned **macOS desktop plus one optional synchronization hub** as an early delivery milestone. That is a reported scope preference, not proof that all four phases' implementation details were finalized.

## 8. Architecture implications (interpretation, not decisions)

- Isolate domain logic (time entries, ownership, grants, adjustments, conflict states) from the desktop UI and hub network layers in the shared Rust core.
- Record object identity, writer identity, provenance, version metadata, and audit information explicitly to support offline replication.
- Design a capability evaluator that can make the same authorization decision regardless of where the request originates.
- Treat conflict resolution as an auditable domain operation that references both competing revisions rather than an invisible overwrite.
- Separate immutable contributor source facts from mutable or append-only owner interpretations.

These are implementation recommendations consistent with the recovered goals, **not evidence of exact decisions made in the missing conversation**.

## 9. Reconstructed decision ledger

| Area | Status | Decision / state |
|---|---|---|
| Shared business logic | Reported later decision | Rust core used by desktop and hub |
| App topology | Reported later decision | Separate desktop and hub applications |
| Offline-first | Earlier established | Standalone app works without hub/internet |
| Owner count | Reported later decision | Exactly one project owner |
| Transfer | Reported later decision | Ownership transfer is atomic |
| Rate authority | Earlier established; reinforced later | Owner-only |
| Project deletion | Reported later decision | Owner-only |
| Delegation | Earlier established; refined later | Project-level capabilities, roles, per-person overrides |
| Project managers | Reported later refinement | May manage collaborators within delegation; no implicit owner powers |
| Contributor source entries | Earlier established | Contributor-controlled; owner does not overwrite originals |
| Owner adjustments | Earlier established; reinforced later | Separate amendments, owner reporting view |
| Sync filtering | Reported later refinement | Deliver only information authorized for recipient |
| Conflict experience | Reported later decision | Explicit manual, Git-style selection on conflicting edits |
| Underlying conflict algorithm | Open | Not determined by user-facing manual resolution decision |
| Framework/UI/database | Open | Not determined by available material |

## 10. Recommended next specification session

Define the precise **capability matrix and data-authority model**, then work through examples of concurrent updates to a time entry, a revoked collaborator, and an ownership transfer while one device is offline. Those scenarios will reveal the requirements for a concrete synchronization protocol without selecting one prematurely.

---

**Preservation note:** This document is a best-effort reconstruction from earlier saved architecture decisions and later conversation summaries. It must **not** be mistaken for a recovered original Markdown block, exact transcript, or independently verified record of the details of the missing exchange.
