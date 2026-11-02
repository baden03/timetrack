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

Each scenario states the expected result, then shows what the protocol must do to produce it. The walk-throughs below assume the offline-authority split in §6.2. Terms: *rev* is a record's revision, *seq* is the hub's sequence number for an accepted operation, *epoch* is defined in 5.3.

## 5.1 Concurrent edit of one entry on two devices

**Setup.** Doug has a laptop and a desktop. Entry E is at rev 7 on both, synchronized. Both go offline.

**Case A, disjoint fields.** The laptop changes the description. The desktop changes the billable flag.
- Expected: both edits survive. E becomes rev 9 with both changes. No question is asked.

**Case B, same field.** Both devices change the description.
- Expected: neither edit is lost. The first accepted stays current (concept §13.3). Doug sees both versions side by side and chooses.

**Case C, related fields.** The laptop moves the start time, the desktop moves the end time.
- These are different fields but one fact, the duration. Merging them could give a result neither device showed.
- Expected: treated as a conflict, like Case B.

**Walk-through.**
1. Each device applies its edit locally and queues an operation carrying `base_revision: 7`.
2. The laptop reconnects first. The hub validates (author is the writer, project open, timing consistent), assigns seq 41, and E becomes rev 8.
3. The desktop reconnects. Its operation has base rev 7 but E is at rev 8. The hub's core lists the fields changed by operations after rev 7 and compares them with the fields in the incoming operation.
4. Disjoint, so the hub applies it as seq 42, E rev 9.
5. Overlapping, so the hub does **not** apply it. It stores a **conflict record** that references rev 8 (current) and the rejected-for-now operation, and sends it to both devices. E stays at rev 8.
6. Either device shows the conflict. Doug picks one version, or writes a combination. That is a new operation based on rev 8, flagged as a resolution, and it goes into the change history with both versions referenced.
7. Until resolved, the desktop shows its own value locally and marks the entry as needing attention. Other readers see rev 8.

**Rules this exposes.**
- Merge granularity is not per column. Fields that must stay consistent form a **field group** that merges as a unit. Timing is one group: start, end, duration, zone, offsets, and the split-group span (a timing edit on one piece of a split group applies to the whole group, concept §8.5).
- An owner's amendment never conflicts with the author's edit. Amendments are separate, append-only records (concept §11), so concurrent edits and amendments both stand and the effective value is recomputed.
- Resolution is idempotent. If Doug resolves on both devices before they sync, the second resolution is based on a stale rev and the hub reports that it is already resolved.
- A third edit made while the conflict is open is based on the local value. The hub treats it as a further operation against rev 8 and may extend the conflict instead of creating a second one.

**Decided.** An entry with an unresolved conflict cannot be marked as invoiced. A contributor invoice is refused until the member resolves their conflicts. A client invoice excludes such entries and lists them, so the owner is never blocked (5.4).

**Field groups (proposal).** Two operations conflict when they change the same group. Changes to different groups merge.

| Group | Fields | Notes |
|---|---|---|
| Timing | entry kind, start, end, duration, time zone, start and end offsets, time-zone database version, split-group span and piece index | One unit, because the values must agree (§8 of the concept). On a split group the unit is the whole group. A running timer's start and stop belong here. |
| Assignment | project, category | Category belongs to a project, so changing the project resets the category. Moving to another project is also checked against the author's capabilities there. |
| Description | description | Required; never empty after a merge. |
| Billable | billable flag | Changes the value, not the time, so it is independent of Timing. |
| Tags | core tags, personal tags | The one exception: merged per tag. Adding tag X and adding tag Y both stand. Adding and removing the same tag is a conflict. |
| Lifecycle | archived state | Archiving does not conflict with edits; the edit applies and the entry stays archived. |

Not editable and so never in conflict: entry ID, author, device, origin, creation record, version history. Hard delete is different from archiving: it wins over a concurrent edit, and the edit comes back as a refused draft saying the entry was deleted. Rate and value are never stored on the entry, so they have no group. The one-timer-per-user rule is reconciled between devices at sync (concept §9), not as an entry conflict.

Billable stands alone (decided). To confirm: whether Assignment should be two groups (a project change and a category change rarely clash).

## 5.2 Revoked collaborator with queued offline work

**Setup.** Steffen is a collaborator on project P. On Monday he records three entries offline on his laptop. On Tuesday the owner removes him (online). On Wednesday his laptop reconnects.

**Expected result (per §6.2).**
- Steffen's earlier accepted entries stay in P. Authorship and history are unchanged.
- The three queued entries are **rejected** with a clear reason. They are not deleted. They stay on his device as drafts with their original data, visible and exportable.
- He receives the removal event. P shows as "access removed". His cached copy remains, since revocation cannot recall data (concept §4.5), and he gets no further updates.

**Walk-through.**
1. The removal is an authority-sensitive operation, accepted at seq 100. It is in the project's log, so every client of P eventually learns it.
2. The laptop connects and uploads its queue. The hub validates each operation in receipt order against the **current** state: Steffen is not a member at seq 100, so each operation fails.
3. The hub does not store the entries in P. It sends back a rejection per operation with a reason code (`not_a_member`) and the seq at which membership ended.
4. The laptop moves the three operations to **drafts**, shows the reason, and stops syncing P.
5. If the owner re-adds Steffen later, the drafts can be re-submitted with their original operation IDs. The hub treats them as new, and they are validated against the state at that time.

**Race.** If Steffen's laptop had reached the hub before seq 100, the same entries would have been accepted. The outcome depends on receipt order. This is the stated cost of receipt-time validation.

**Rules this exposes.**
- A rejection is data delivered to the originating device, not a transport failure. The queue must tell "retry later" apart from "refused for good".
- The hub never forwards rejected work to the owner. Doing so would be the hub acting for Steffen.
- The client needs a stable idea of "this project is no longer mine" so it stops queuing project work and keeps the local copy readable.

**Decided: pending removal.** A member with open contributions (accepted entries not yet closed on a contributor invoice) is not removed at once. They are set to **pending removal** until those contributions are closed, and become removed automatically at that point. The steps above describe a member with nothing open; otherwise step 1 sets pending removal instead.
- While pending removal: the member can still view and edit their open entries, resolve conflicts on them and close a contributor invoice. They cannot record new work. Operations that create new entries are refused like those in the walk-through, and stay as drafts.
- The owner sees the pending state and who it is waiting for. Pending removal is an epoch change.
- A removed member with nothing open is removed at once, as in the walk-through.

**Decided: forced removal.** The owner can always force a removal, with no waiting period. The member's open entries stay in the project, uninvoiced and flagged, visible to the owner, and the audit record shows the forced removal. The member's queued work is then refused as in the walk-through.

Rejected drafts are not surfaced to the owner (re-adding covers it). A downgrade of role instead of removal (scenario 5) uses the same rule: operations needing a lost capability are refused, operations needing a kept capability are accepted.

## 5.3 Ownership transfer with an offline device

**Setup.** Alice owns project P. She starts a transfer to Bob and chooses to stay as Project Manager afterward. Alice also has a laptop that went offline before the transfer. It holds three queued operations: one of her own time entries, one amendment to Bob's entry, and one new rate rule.

**Expected result.**
- The transfer executes once, atomically, and only after Bob accepts. P always has exactly one owner.
- Alice's own entry is accepted. Authorship and history are unchanged.
- The amendment and the rate rule are refused. They need owner capabilities, and Alice is now a Project Manager. They stay as drafts on her laptop.

**Walk-through.**
1. Alice's request is an authority-sensitive operation and needs the hub. The hub records it as a **pending transfer**: proposed owner, chosen post-transfer status, and the current epoch.
2. P is **not frozen** while the transfer is pending. The current owner keeps working, and owner-only operations are accepted as usual until the transfer executes. Execution is a single step, so no pause is needed. Proposal to confirm: Bob's acceptance names the epoch he reviewed, and if owner-level state changed since (members, rate rules, clients), the hub asks him to review again before executing.
3. Bob accepts, online, from any device. The hub executes in one step: owner becomes Bob, Alice's status becomes Project Manager, the epoch increases, and the audit record is written with Alice's chosen status. This is seq N.
4. Every client learns seq N from the log.
5. Alice's laptop reconnects. Each queued operation is checked at receipt. Her entry is accepted. The amendment and rate rule need owner capability, which Alice no longer holds, so they are refused with `ownership_changed` and seq N. The epoch recorded with them explains why.
6. They remain drafts. Nothing is forwarded to Bob. If Alice wants them applied, Bob decides, as owner, to create them himself.

**Epoch.** A project has an **authority epoch**, increased by every ownership or role change. Each authority-sensitive operation records the epoch it was made under. The hub decides by the capability held at receipt, not by the epoch alone (see 5.5). The epoch lets a refusal say what changed and when. It also covers scenarios 2 and 5.

**Rules this exposes.**
- Acceptance cannot happen offline. Bob must be online to accept, which is also why the former owner cannot be left half-transferred.
- If Alice is not staying (Leave Project), her queued own entries are refused like Steffen's in 5.2.
- Pending transfers need an expiry or an explicit cancel, so a request does not linger. Nothing else depends on it while it is pending.
- If Alice stays, Bob must set Alice's base rate before accepting (open item #26 in the concept), otherwise her post-transfer entries have no rate.

**Decided (what moves with ownership).** Everything that belongs to the project moves with it (clients, categories, shared tags, membership, existing rate rules, invoices already issued, locks). Alice's own entries and contributor invoices do not move; they stay hers. The project's currency stays fixed at transfer, so no retroactive change to figures. The new owner then defines new rate rules going forward.

## 5.4 Invoice close racing an offline edit

**Setup.** Doug and Alice are members of P; Alice owns it. On 3 November Doug's laptop is offline. It holds an edit to entry E (12 October): the description changed and the duration raised from 2:00 to 2:30. Alice closes the client invoice for October from her own device.

**Expected result (concept §12.7).** The edit made offline before the close but received after it is refused. E is shown as invoiced. Doug keeps the change as a visible draft and can ask for a correction.

**Walk-through.**
1. Alice's dialog shows Doug's status and last sync time (concept §12.5). She closes anyway. The hub assigns seq 200, pins every accepted entry in the period at its current version with minutes, rate and value, and locks them. E is pinned at the version the hub knows.
2. Doug's laptop reconnects. The hub validates the queued edit against E: locked by the client invoice at seq 200. It refuses it with `entry_locked`, naming the lock and its kind.
3. The laptop shows E as "Locked: invoiced" and the edit as a draft with the reason. Doug's remedies are Alice's: she may add an **amendment** (applied as an adjustment in the next period, concept §12.4), or reopen the client invoice with a reason, after which the draft can be re-submitted under its original ID. Nothing else in the lock changes.
4. New October entries on the same laptop are accepted. They are uninvoiced and go on the next invoice (concept §12.3).

**Variations.**
- *Contributor close instead of client close.* Doug closes his own contributor invoice on his phone while the laptop is still offline. The same refusal applies. He can reopen his own invoice himself, unless a client invoice now includes the entry.
- *Two closes at once.* Doug's contributor close and Alice's client close race. The two locks are independent facts (concept §12.3), so both succeed in receipt order. Each pins whatever version is current when it is received. An edit accepted before a close is in the snapshot; one received after is refused.
- *Date edits across the boundary.* Moving an unlocked entry into a closed period is allowed; it is uninvoiced and goes on the next invoice. Moving a locked entry out of its period is refused.
- *Split group on a locked boundary.* The locked piece is frozen; the other piece can be edited but the midnight boundary cannot move (concept §12.8).
- *Running timer in the period.* The entry has no end yet. Recommended: the close excludes it and lists it as "still running". Once stopped it is uninvoiced and goes on the next invoice.

**Decided: conflicts and invoicing.** Concept §13.3 used to say that if an invoice closes while a conflict is open, the entry is pinned at the accepted version and the held change is rejected. The decision recorded in 5.1 says conflicts must be resolved before an entry can be invoiced. Taken literally, that lets a member hold up the owner's client invoice, since only the author can resolve their own conflict (concept §13.3) and the author may be offline. Decision:
- *Contributor invoice:* refused while the member's own entries have an open conflict. The member controls this.
- *Client invoice:* entries with an open conflict are **excluded** from the close and listed, and go on the next invoice. The owner is never blocked by someone else's unresolved edit.

Concept §12.2 and §13.3 now say this.

**Rules this exposes.**
- A lock is a hub-side fact with a seq and a kind. A device learns it on its next sync. Offline the app cannot know, so `entry_locked` is the normal path, not an error.
- A refused edit to a locked entry can be re-submitted after a reopen without re-basing, since the entry cannot have changed in between.
- Amendments are not blocked by locks. They are separate records.

## 5.5 Role change while a device is offline

**Setup.** Steffen is a Project manager on P with `invoices.close` and `entries.viewAll`. While his laptop is offline, Alice changes him to Collaborator. The laptop holds three things: one of his own entries, a pending request to close a client invoice, and a cached copy of all members' entries and rates.

**Expected result.**
- The own entry is accepted. Collaborators record time.
- The invoice-close request is refused, because he no longer holds `invoices.close`. It stays as a draft with the reason.
- He stops receiving other members' entries and rates, and the app removes what it no longer entitles him to.

**Walk-through.**
1. The role change is accepted at seq 120. It is delivered to Steffen's devices as an entitlement change.
2. His laptop reconnects and uploads. Each operation is checked against his capabilities **at receipt**. The entry needs nothing beyond what a collaborator has. The close needs `invoices.close`, which he lacks, so it is refused with `capability_lost` and seq 120.
3. The hub sends the entitlement change before any further data. The client drops the data it can no longer show, and the needs-attention list shows the refused request.
4. Nothing has to be undone downstream. An authority-sensitive operation never takes effect offline (§6.2), so a refused one has produced no effects elsewhere.

**Role raised instead.** Doug is promoted from Collaborator to Project manager while offline. His laptop lacks the members' entries he is now entitled to. On reconnect the hub sends the entitlement change followed by a **backfill** of everything newly visible. This is the same mechanism as a new device (concept §13.7), scoped to one project.

**Rules this exposes.**
- Removal of data the person is no longer entitled to is a courtesy, not security (concept §4.5). The hub never sends data the person is not entitled to.
- An entitlement change needs its own event type in the log, ahead of the data it governs.
- Per-person overrides on a preset (concept §4.3) behave the same way: the capability set at receipt decides.

**Correction to 5.3.** The first version of the epoch rule there refused any operation made under an older epoch. That would refuse Bob's valid work whenever someone else's role changed. The decision is now the capability held at receipt; the epoch is recorded so a refusal can say what changed and when.

## 5.6 Joining a hub from a standalone installation

**Setup.** Doug has used a standalone installation for months. He owns two projects: P1 (client work, 2,000 entries, two closed invoices) and P2 (private). He has personal tags and preferences. Alice, who runs the hub at twinpictures.de, sends him an invite. He also receives a collaborator invite to Alice's project Q.

**Expected result.** Doug joins, chooses to bring P1 but not P2, and ends up with his history, locks and invoices intact on the hub. He also sees Q. P2 stays local. He can later disconnect and return to standalone with his data (concept §13.5).

**Walk-through.**
1. He opens the invite link and signs in with a passkey on the hub's page. The hub creates the account with a handle, and **claims** the install's local identity: the local user ID becomes the hub user ID. Authorship on existing entries needs no rewriting. This is why identifiers are client-generated and never reused.
2. He chooses P1. The app first runs a **preflight**: the same validation the hub will apply, locally, listing anything that would be refused.
3. P1 uploads as a bulk import. It is idempotent and resumable by operation ID, and the hub applies it as a **single unit per project**: all of P1 enters or none, since a project with locked invoices that point at missing entries would be inconsistent. P1 becomes visible to anyone else only after the import completes.
4. The hub validates structure, as for any arrival (concept §13.6): authorship, time rules, split-group consistency, and the future-entry check. It does not recompute invoice snapshots or splits; it imports what was recorded. Existing locks and snapshots are imported as facts.
5. History records are imported with the data, so the audit trail stays complete.
6. Q is pulled by the normal entitlement-driven sync. Personal tags remain personal. A personal tag with the same name as a core tag is mapped to the core tag on import (decided).
7. If he later disconnects, the hub keeps its copy and the install keeps working standalone. Reconnecting later is ordinary sync, not a second import.

**Rules this exposes.**
- Import is its own protocol mode: bulk, per-project atomic, resumable, with preflight.
- Imported records keep their IDs, authorship, device and origin metadata.
- Identity claim must be collision-checked on the hub, and an alias is an audited event.

**Decided.** Import is all-or-nothing per project, after a preflight check. The alias step is part of the first milestone (see 5.9).

## 5.7 Time-zone data drift

**Setup.** Doug records 23:00 on 24 October to 04:00 on 25 October in Berlin, the night the clocks go back (concept §18 test vector: 1:00 + 5:00, real duration 6:00).

**Walk-through (the vector).**
1. His client splits at local midnight. Piece 1 runs 21:00Z to 22:00Z with offset +02:00. Piece 2 runs 22:00Z to 03:00Z, starting at +02:00 and ending at +01:00. Each piece stores its UTC instants, zone, the offsets in force at its start and end, and the time-zone database version of the client.
2. The hub checks structure only: the pieces are contiguous in UTC, cover the whole span, the minutes add up (60 + 300 = 360), the boundary falls at local midnight under the recorded offsets, the offsets are plausible, and the group is complete. It needs no time-zone database.
3. Steffen reads the entry in another zone. Dates in reports follow the author's recorded dates. Nothing moves (concept §8.5, the Costa Rica example).

**Drift cases.**
- *Rules change after recording* (a country alters or abolishes DST). The recorded offsets are facts. Nothing is recomputed, so dates in old entries never move.
- *Two clients with different tz data.* A laptop with newer data edits the description, a desktop with older data edits the end time. These are different field groups (Description and Timing), so they merge. If both edit Timing it is a normal conflict, and each side's tz database version is shown for information.
- *Editing the timing of an old entry.* The editing client recalculates the whole split group with its current data and sends the new group as one operation. The hub validates structure only. Pieces already locked stay frozen (concept §12.8).
- *Travelling.* Recording in New York while based in Berlin stores the zone in force at recording. The hub's plausibility check passes. Others see the author's recorded zone by default (open item #34).
- *Wrong device zone.* A Mac set to the wrong zone produces a self-consistent entry. The hub cannot detect it; the author corrects it by editing, which recalculates.

**Rules this exposes.**
- The time-zone database is never part of sync. Offsets and the version are stored; nothing else is shared.
- The hub's validator is pure arithmetic over recorded values, so client and hub share one implementation and the concept's test vectors run against it.
- The core receives time-zone data as an input from the host (bundled or OS, open item #36), which keeps its tests deterministic.

**Decided.** When someone edits the timing of an entry, the new times are interpreted in the entry's own recorded zone, not the zone the device is in now. Changing the zone is a separate, explicit action.

**Travelling across zones within a day.** A person may record in one zone in the morning and another in the evening. The model already copes, because hours never depend on the zone: duration is the difference between two UTC instants. A zone only decides which calendar date an entry belongs to and where a midnight split falls. So the rules concern dates, not hours.

1. *Each entry has its own zone.* Morning in Berlin and evening in New York give two entries with two zones. A day's total is the sum of its entries by their own local dates.
2. *Overlap is judged in UTC.* Entries from different zones cannot falsely overlap or leave false gaps.
3. *Entering time after the fact.* The zone field defaults to the device's current zone, but a morning's work entered that evening in another zone may belong to the earlier zone. The entry shows its zone next to its times whenever it differs from the entry before it or from the device's zone, and one click changes it. Typed times are read in the chosen zone.
4. *A timer running across a zone change* (a flight, a train). The app notices the system zone changed, and splits the running entry at that instant into linked pieces, each with its own zone and offsets. No prompt is shown, because the stale-timer prompt is the only notification (concept §9). The pieces appear as one group in the day view.
5. *A manual entry that crosses a zone change.* One entry has one zone. Hours are still right. Only the date at the boundary could be off by a day, and the person can enter two pieces to be precise.
6. *Reporting and invoice periods* use each entry's own local date. An entry at 23:30 on 31 October in New York and one at 07:00 on 1 November in Berlin are close in real time but fall in different months. This follows the rule that reporting never moves dates (concept §8.5).
7. *Not offered:* a setting that attributes every day to a fixed "home" zone. Changing it would move dates, which breaks locks and snapshots. A display-only option may come later and would never change stored dates.

**Hub validation for zone changes.** Midnight is not the only legitimate piece boundary now. Each piece records why it begins where it does: `midnight` or `zone_change`. For a midnight boundary the hub checks local midnight under the recorded offsets, as before. For a zone change it checks that the pieces are contiguous at the same UTC instant and that the zone differs. Offsets must be plausible in both cases. Still no time-zone database on the hub.

**To confirm.** Items 3 (default zone and visible chip), 4 (silent automatic split on a zone change) and the boundary reason as part of the concept's §7.7 and §8.5. This also needs a new open item in the concept.

## 5.8 What the walk-throughs require of the protocol

- **Operations** carry: operation ID, author, device, type, target, base revision, changes, and, if authority-sensitive, the epoch.
- **The hub log** has a sequence number per project, so clients and the hub agree on the order of accepted operations, including membership and ownership events.
- **Three outcomes** for an uploaded operation: accepted (with seq), conflicted (stored as a conflict record), refused (with reason code and the seq that explains it). Transport failure is a fourth, separate state.
- **Client queue states:** pending, accepted, needs attention (conflict), draft (refused).
- **Field groups** are defined in the core, shared by client and hub.
- **Filtered delivery.** Removal and downgrade events must reach the affected person while later project data does not.
- **Re-submission** of a refused operation under its original ID is allowed and idempotent.
- **Member states** include pending removal, in addition to active and removed. The core evaluates capabilities per state.
- **Invoice close** is refused while any entry in the period has an open conflict, and reports which.
- **No freeze.** Pending transfers do not change what operations are accepted. Only execution does.
- **Locks** are hub-side facts with a seq and a kind (contributor or client). They are independent of each other. Refusals name the lock.
- **Entitlement changes** are log events delivered ahead of the data they govern. Widening an entitlement triggers a scoped backfill. Narrowing triggers a best-effort local purge.
- **Capability check at receipt.** Authority-sensitive operations are accepted if the capability is held when received. The epoch is recorded for explanation.
- **Bulk import** is a separate mode: preflight, per-project atomic, resumable by operation ID, preserving IDs, authorship, device and origin.
- **Identity claim and alias.** Joining claims the local identity; a second install links through an audited alias.
- **No time-zone data in sync.** Recorded offsets and the database version travel; the validator is arithmetic and shared by client and hub.
- **Piece boundary reason** (`midnight` or `zone_change`) on every split piece, checked by the hub with the rule that fits the reason.
- **First-run choice.** The install records whether it is stand-alone or connected, and can move between them.

## 5.9 First run: stand-alone or part of a network

This is product behavior. Once confirmed, the concept needs a first-run section (§13.5 and §14).

The first launch of any install asks one question: **how will this install be used?**

| | Choice | Needs a hub | Who it is for |
|---|---|---|---|
| **a** | **Stand-alone.** This Mac only. | No | One person, one device, or one person who does not want to sync |
| **b** | **Part of a network.** Sync with my own devices (local sync) and/or work with other people (collaboration). | Yes | Everyone else |
| b1 | Join an existing hub | Yes | Someone who has an invite or a hub address |
| b2 | Create a hub | Yes | The person who will run it |

"Local sync" and "collaboration" are the same path. Both need a hub, because there is no direct link between two installs in the first milestone (federation is hub to hub, Phase 4). The difference is only who else has an account on it.

### a) Stand-alone

1. The person chooses stand-alone and gives a display name.
2. The install creates a random local identity (concept §3) and opens the empty app. No account, no sign-in, no network use.
3. Everything works (concept §16), including backup and restore.
4. Nothing in the app asks for a hub afterwards. "Connect to a hub" lives in Settings and leads into b1 or b2 with the existing data (5.6).

### b1) Join an existing hub

1. **Entry.** The person opens an invite link, or types a hub address. An invite carries the hub address and a token. An invite may be for the hub, for a project, or both (concept §4.7).
2. **Check.** The app contacts the hub, checks the protocol version (concept §13.10), and records the hub's permanent ID next to its DNS name. If the name later points at a different hub ID, the app warns instead of connecting silently.
3. **Sign in.** The hub's page opens in the system browser and the person signs in with a passkey: a new account from the invite (handle, display name, email), or an existing account.
4. **Register the device.** The hub issues a device ID. The device name defaults to the Mac's name. The session credential is kept in the macOS Keychain (open item 2).
5. **Data.**
   - A fresh install pulls what the person is entitled to (concept §13.7). A new Mac for an existing account is simply another device and never needs an alias.
   - An install that already holds stand-alone data goes through the import described below.
6. The app shows sync status (concept §13.9) and opens.

**Failure paths.** Hub unreachable: setup waits and offers stand-alone, with "connect later" in Settings. Incompatible protocol: the app says which side must update. Hub ID mismatch: the app stops and explains.

**Joining with existing local data.** Doug has two Macs that were stand-alone and now wants them linked.
1. *Mac 1* connects. The account is created from the invite, the local identity is claimed, and the projects he picks are imported (5.6).
2. *Mac 2* connects and signs in to the **existing** account. The app sees local data and asks which projects to bring in. The ones picked are imported. The rest stay local to this Mac and are never uploaded.
3. For the imported projects, Mac 2's local identity is recorded as an **alias** of the account. Its entries are attributed to the account. The original identity stays in the entries' provenance, and the hub log records an audited "install linked" event.
4. Later Macs skip step 2 if they have nothing of their own.

**Limits for the first milestone.** Projects and clients from the two installs stay separate even if their names match. There is no project merge. Duplicates are archived by hand. Personal tags merge by name. A project merge is a candidate for the concept (§22).

### b2) Create a hub (deep dive)

A hub is a separate application from the desktop app (§1). Creating one is an operator task, so the first question is what "create a hub" means inside the desktop app. It is not decided.

**Constraints from earlier decisions.**
- Passkeys are bound to the web origin (concept §5). The hub's sign-in page needs a stable name and a valid certificate before anyone can sign in. Only `localhost` is exempt.
- The hub has a permanent ID and a DNS name that can change (concept §3).
- The hub operator uses the server command line: create an account, issue an invite, update an email, disable or enable an account (concept §5). There is no in-app admin role.
- The hub sees plaintext data. It needs TLS, access control, disk encryption and encrypted backups (concept §6).

**Who creates hubs.**
1. A person who only wants their own devices synced. They want minimal ceremony.
2. A small team on a server they manage (for example Twinpictures, Doug, Steffen).
3. Later: organizations and federation.

**Where a hub can run.**

| | Option | Gain | Cost or open point |
|---|---|---|---|
| A | A hub on a server with a domain (VPS or own machine). Operator starts the hub, TLS is automatic or via a reverse proxy | Matches the design. Reachable from anywhere. Passkeys work | Needs a domain and some server skill |
| B | A hub on an always-on Mac or a small computer at home | No hosting cost | Reaching it outside the home network (forwarding or tunnel). A real certificate for a local address. A sleeping computer is an outage |
| C | "Create a hub on this Mac" from the desktop app: the app starts a hub process, local network only | One click for local sync | Available only while that Mac is awake. Passkeys do not fit a local address. Needs a different sign-in (device pairing by code or QR) |
| D | A hub hosted by someone else | Easiest for users | Outside the first milestone. Raises trust and operations questions |

**What "Create a hub" does in the app (proposal).** The app does not deploy hubs. It shows what is needed (a server, a domain, one command), links the setup guide, then waits for the operator's hub address and first invite and continues as **b1**. The deliverable is the hub itself and a one-command setup, not a deployment wizard. This keeps the desktop app small and matches the separate-applications decision.

**Bootstrap sequence for Option A.**
1. Operator starts the hub with a domain name. The hub generates its permanent ID and obtains a certificate.
2. Operator runs the command line to create the first account. The command prints an invite link.
3. Operator opens the link on their Mac and joins as in b1. Others join with further invites.

**Questions for the deep dive, in order.**
1. *Who and where.* Is A enough for the first milestone? Is C wanted later for people who only need local sync?
2. *Sign-in without a domain.* Do we support only passkeys (so a domain is required), or also device-held keys with pairing for personal hubs (concept §5 alternative)? Two methods mean more to build and secure.
3. *Packaging and storage.* One program with embedded storage and no database server would keep a hub small. It ties to open item 3 and to the PowerSync finding that its service needs two databases (§7.1).
4. *TLS.* Certificates handled by the hub itself, or by a reverse proxy?
5. *Command line.* The exact commands for the operator: accounts, invites, disabling, backup, restore, upgrade, rename.
6. *Backup, restore and upgrade.* What the operator does, and what happens to clients when the protocol version rises.
7. *Moving or renaming.* The DNS name can change (concept §3). What happens to existing installs, invites and passkeys when it does?
8. *One hub per install.* The first milestone allows an install to be connected to one hub. Belonging to several hubs is Phase 3.

**Recommendation.** Support A in the first milestone with a one-command setup and a reverse proxy guide. Document B as advanced. Defer C (it needs a second sign-in method). Leave D out. Treat the questions above as the agenda for the next discussion.

### Changing later

- Stand-alone to network: Settings, connect, then b1 or b2. Existing data follows 5.6.
- Network to stand-alone: disconnect. The install keeps its data. The hub keeps its copy and any alias stays in the history (concept §13.5).
- A disabled account or a revoked device stops syncing. The install keeps its local data and can continue stand-alone (concept §13.8).
- Moving to a different hub: not supported in the first milestone beyond disconnect and join again. Open.

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
