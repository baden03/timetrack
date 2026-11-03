# TimeTrack — Product Concept v2

**Status:** Draft for review. Technology stack, platforms and implementation plan are not yet decided.
**Date:** November 2026
**Supersedes:** Concept v1, the centralized multi-user design, archived at [docs/spec-v1-centralized.md](docs/spec-v1-centralized.md) (git tag `spec-v1-centralized`).
**Background:** [docs/timetrack_architecture_decisions_2026-11-02.md](docs/timetrack_architecture_decisions_2026-11-02.md) records the brainstorming this concept builds on. Where the two differ, this document is current.
**Architecture direction:** [docs/architecture.md](docs/architecture.md) (shared Rust domain core, separate desktop and hub applications).
**Rollout plan:** [docs/plan.md](docs/plan.md) (phases, release scope and acceptance criteria). Where it moves a feature to a later release, the plan is current.
**Translating:** [docs/translating.md](docs/translating.md) explains how to add the interface in another language.
**Also archived:** the 2024 prototype reconstruction, [docs/brief-v1.md](docs/brief-v1.md).

This document is technology-agnostic. It states what the product does and the rules it follows, not how it is built.

---

# 1. Purpose, Principles and Scope

## 1.1 What TimeTrack is

TimeTrack is a local-first, open-source time-tracking system. A standalone installation is fully useful on its own, with no account and no network. Optionally, an installation connects to a **hub** to synchronize a person's devices and to collaborate on shared projects.

The first practical goal is to replace the per-client, per-project Google Docs currently used to record hours: collaborators log their own time, and the project owner sees it without having to ask.

## 1.2 Principles

1. **Standalone first.** Every core feature works with no hub and no internet connection.
2. **Every installation can own projects.** There is no privileged primary installation.
3. **The hub is optional.** Connecting to one enables device sync and shared-project collaboration. Standalone use never depends on it.
4. **Open source and independently operable.** Anyone can run a hub. There is no global master hub.
5. **One writer per record.** Every record has exactly one author with authority over it. Others refer to a record with their own records, never by rewriting it.
6. **Originals are never rewritten by someone else.** A collaborator's entry stays under the collaborator's control. The owner records adjustments separately.
7. **The hub arbitrates what only a hub can.** Permissions, visibility and the order of locks are decided at the hub, not by clients.
8. **Design for the later phases now.** Identifiers, identities and the sync protocol are shaped so that independent hubs and federation need no rewrite.

## 1.3 Design philosophy

Recording and correcting time must take as little effort as possible. A user should be able to open the app, pick a project, enter or start a time record, and see the day's total immediately. Administration stays out of the way.

**Priorities: speed, simplicity, accuracy, minimal interruption.**

## 1.4 Phases

| Phase | Content |
|---|---|
| **1. Standalone** | A local application. No hub, no account, no network. |
| **2. Single hub** | Sign in to one hub to sync devices and collaborate on shared projects. Changes for offline participants are queued and delivered on reconnection. |
| **3. Independent hubs** | Multiple operators run their own hubs. Users and devices belong to different hubs. No hub is subordinate to another. |
| **4. Federation** | Trusted hubs exchange data so people on different hubs can collaborate. |

## 1.5 First milestone

The first usable release is **Phase 1 plus Phase 2**: the standalone app, then one hub. Build order is standalone first, hub and sync second. Standalone is usable throughout.

**In the first milestone:**

- Standalone local app with all single-user features (§8–§12, §14–§18)
- Hub accounts, sign-in, device sync (§3, §5, §13)
- Shared projects with invitations, roles and capabilities (§4)
- Owner-defined rates and effective-dated rate history (§10)
- Entry versions and owner amendments (§11)
- Contributor and client invoice locks (§12)
- Backup and restore (§16)
- Translatable interface, English by default (§17)
- A hub operator command line

**Not in the first milestone:**

- Independent hubs and federation (Phases 3 and 4)
- End-to-end encryption
- Paid tiers and hosted-hub services
- A public API (designed for, not built; §20)
- A reporting module and invoice documents
- General data import and export (backup and restore only)
- Email notifications
- Mobile apps and a web client

**Designed in now so later phases need no rewrite:** identifiers that carry their origin hub; `name@hub` identities; tombstones for deletions; ordering of one person's edits across devices; versioned entries; a versioned sync protocol.

**Delivery in releases.** The first milestone is delivered in stages (see [docs/plan.md](docs/plan.md)). Release 1 is the standalone app, with its own acceptance criteria, plus a **monthly summary**: for a month, per client and project, by category, the total hours and the list of descriptions, copyable as text. Release 2 adds one hub with Owner and Collaborator roles. Release 3 adds invoice locks on hours. These move to later releases: owner-defined rates, rate rules, entry values and rate conflicts (§10), which matter only once a reporting module exists, because the billed rate lives in the invoicing software; owner amendments (§11); the Project manager preset and per-person overrides; ownership transfer; and the automatic split on a time-zone change. Sections 4, 10, 11 and 12 describe the complete design, not the content of the first release.

## 1.6 Primary platform

Desktop, macOS first. Windows and Linux are intended but later. Mobile is not in the first milestone, and nothing in the model may assume desktop-only. Platform and technology choices are the next topic and are open.

---

# 2. Decisions Log

| Topic | Decision |
|---|---|
| Product direction | Local-first, open source. Standalone fully works with no hub or network. |
| First milestone | Local app plus one hub. Standalone first in build order. |
| Hubs | Optional. Independently operable. No master hub. Federation is last. |
| Hub identity | A stable internal hub ID plus a changeable DNS name. |
| User identity | `handle@hub`. Underneath, a permanent opaque user ID. Unique email per hub account. A standalone install has a random local identity, claimed by a hub account when the user joins. |
| Trust | The hub sees plaintext, protected by TLS, access control, disk encryption and careful logging. No end-to-end encryption in the first milestone. |
| Project ownership | A single owner per project. |
| Record authority | Collaborators own their entries. The owner owns the project, clients, categories, shared tags, members, rates, amendments and client invoices. |
| Collaborator entries | The owner can view them but never rewrite them. |
| Owner corrections | Separate amendment records that affect the owner's figures only. |
| Permissions | Project-level capabilities. Roles are presets. Per-person overrides. Default deny, no privilege escalation, one evaluator everywhere. |
| Ownership | Exactly one owner per project at any time. Transfer is atomic, needs explicit acceptance, and the former owner chooses their own post-transfer status (project manager, collaborator, or leave). History is permanently audited. |
| Owner-only operations | Defining rates, deleting the project, transferring ownership. Not delegable through roles or overrides in the first milestone. |
| Architecture | A shared Rust domain core used by separate desktop and hub applications (see `docs/architecture.md`). |
| Visibility | A collaborator sees only their own entries and rates. Project managers and owners see all entries. |
| Rates | The project owner defines each member's hourly rate. Immutable, effective-dated, with overrides by client, project and category. |
| Currency | Per owner, default EUR, locked once any rate exists. Totals in different currencies are never added together. |
| Time recording | (a) start and end, (b) duration only, (c) live timer. |
| Display of durations | `hh:mm` everywhere. |
| Precision | Whole minutes. No rounding. Money is rounded half-up to the smallest currency unit once per entry. |
| Midnight | Entries crossing midnight are split automatically into linked entries. The author's client performs the split when the time is recorded and stores the result with its timezone context. The hub stores and checks it for consistency but never recalculates it. |
| Timezone | One IANA timezone per user preference, captured on every entry together with the UTC offsets in force. Recorded interpretation is authoritative; viewing in another zone never moves an entry's date or duration. Client and hub need not share timezone data versions. |
| Description | Required. |
| Project date | Derived from the first and last entries. |
| Invoicing | Locks attach to a closed invoice period, not the calendar month. Two kinds: contributor invoice and client invoice. Invoice documents stay outside TimeTrack. |
| Late entries | Allowed after a period closes. They are uninvoiced and bill next time. |
| Reopening an invoice | Allowed, with a mandatory reason, audited, visible to the other party. |
| Offline | Supported in standalone and for temporarily disconnected hub users. |
| Conflicts | By design rare. The hub orders locks and permissions. Competing edits to one record are shown to the person, who chooses which version to accept, in the manner of Git. Never silent last-write-wins. Rejected offline changes are kept visible, never dropped. |
| Editing | Row-level in-place editing. |
| Delete | Archive by default. Danger-zone hard delete where permitted. Invoiced entries are never deleted. |
| Sign-in | Passkeys through a minimal hub-served page, opened in the system browser. No sign-in in standalone. |
| Hub operator | A command line on the hub server. No web login. |
| Translation | All interface text is translatable, English by default. All code and comments are in English. |
| API | A future add-on, designed for from the start. |
| Notifications | In-app only. |
| Timer | One running timer per user. |
| Entry window | Default view shows the last 60 days (a preference). Search covers everything the user may see. |
| Tags | Shared tags managed by the owner, plus personal tags any user can create by typing them. Visible to the owner and project managers where the entries are visible. Names up to 40 characters, up to 10 per entry. |

---

# 3. Identity, Hubs and Devices

## 3.1 Local identity

A new installation creates a random local identity and a local database. No account is needed. The installation is the owner of everything created in it.

## 3.2 Hub

A hub has a permanent internal **hub ID** and a current **DNS name** (for example `time.twinpictures.de`). The name is a namespace for the hub's users and must be stable. Keeping the ID separate allows a later move to a new name.

## 3.3 Account

A hub account has:

- a permanent, opaque **user ID**
- a **handle**, unique within the hub, shown as `handle@hub`
- a **display name**
- an **email address**, unique within the hub, used for invitations and recovery, not part of the identity and changeable
- a status: active or disabled

The same email may have accounts on different hubs.

**Claiming.** When a standalone user joins a hub, the local identity becomes linked to the new account. Existing local data keeps its identifiers and remains private until the user chooses to sync or share it.

## 3.4 Devices

A person may use several devices. Each has a device ID. Devices of one person converge through the hub. A device can be revoked, which stops its future sync (§13.8).

## 3.5 Identifiers

Every record has an identifier that is:

- unique across installations and hubs
- created on the device that creates the record, so no round trip is needed
- stable and never reused
- roughly ordered by creation time
- origin-aware (it names the hub or installation of origin)

---

# 4. Roles and Permissions

## 4.1 Two levels

- **Hub level.** Who may have an account, and hub-wide policy. Managed by the **hub operator** (§5.5).
- **Project level.** What each member can do within a project. Managed by the project owner and those they delegate to.

There is no single "admin" who sees everything. A person is an owner of their own projects, a member of others'.

## 4.2 Project capabilities

Access is a set of **capabilities** on a project. Roles are convenient presets, and any person can have individual overrides.

| Capability | Meaning |
|---|---|
| `entries.viewOwn` | See own entries and own rates (always granted to a member) |
| `entries.logOwn` | Create, edit and delete own entries |
| `entries.viewAll` | See all members' entries |
| `amend` | Create owner amendments (§11) |
| `rates.viewAll` | See all members' rates and values |
| `members.invite` | Invite people to the project |
| `members.manage` | Change members' roles, capabilities and removal |
| `project.edit` | Edit project settings, categories and shared tags |
| `invoices.close` | Close client invoices |
| `invoices.reopen` | Reopen client invoices |
| `project.archive` | Archive and restore the project |
| *Owner-only operations* | Define rates and resolve rate conflicts; delete the project; transfer ownership. Not capabilities and never delegable (§4.8) |

## 4.3 Presets

| Capability | Collaborator | Project manager | Owner |
|---|---|---|---|
| `entries.viewOwn`, `entries.logOwn` | ✔ | ✔ | ✔ |
| `entries.viewAll` | | ✔ | ✔ |
| `members.invite`, `members.manage` | | ✔ | ✔ |
| `project.edit` | | ✔ | ✔ |
| `amend`, `rates.viewAll` | | | ✔ |
| `invoices.close`, `invoices.reopen` | | | ✔ |
| `project.archive` | | | ✔ |
| Owner-only: define rates, delete project, transfer ownership | | | ✔ (not delegable) |

The owner may grant any delegable capability to anyone as an override, for example `invoices.close` to a project manager. Owner-only operations cannot be granted.

## 4.4 Visibility rules

- A collaborator sees only their own entries and only their own rates and values.
- A project manager sees all entries on the project, but not rates unless granted.
- Shared tags, categories and the project's client name are visible to all members.
- Personal tags are visible to their creator, and to the owner and anyone with `entries.viewAll` on entries they can see.
- The restriction applies identically on every device and in every API client.

## 4.5 Enforcement and its limits

- Permissions are enforced by the **hub**, which filters what each person receives and rejects changes they aren't entitled to make. Clients also enforce them for usability, but the hub is the authority.
- Data already synced to a device **cannot be recalled**. Revoking access, removing a member or disabling an account stops future sync only.
- Capability changes are applied by the hub in the order it receives them. A change made offline under a capability that has since been withdrawn is rejected on arrival (§13.6).

## 4.6 Hub policy

The hub operator decides policy for the hub: who may create accounts (default: only the operator), and whether ordinary users may create projects (default: yes). Cross-hub policy belongs to Phase 4.

## 4.7 Invitations

1. An owner or a person with `members.invite` invites an existing hub account to a project by handle, choosing a preset.
2. Until accepted, the invitee sees only the invitation.
3. On acceptance, the project's reference data and the invitee's entitled data are synced to their devices.
4. The owner or a person with `members.manage` can change or remove a member. Removal stops future sync. The former member keeps their own entries as read-only history and cannot add new ones. Their entries remain visible to the owner.

## 4.8 Ownership and transfer

- A project has **exactly one owner** at any time.
- **Owner-only operations** are defining rates (including resolving rate conflicts), deleting the project, and transferring ownership. They are never delegated through roles or overrides in the first milestone. A project manager has none of them, whatever else they are granted.
### Transfer workflow

1. The current owner starts the transfer and selects the proposed new owner, an active account on the same hub.
2. The current owner chooses their **own status after the transfer**:
   - **Project manager:** remains involved with delegated management capabilities.
   - **Collaborator:** keeps access as a regular contributor.
   - **Leave project:** gives up all membership and access.
3. The proposed new owner must **explicitly accept**. They may decline, and the current owner may withdraw the offer until it is accepted.
4. The transfer executes **atomically** at the hub. There is always exactly one owner, never none and never two.
5. The owner-only operations pass to the new owner.
6. Historical authorship is unchanged: time entries, amendments, invoices and history records keep their original authors.
7. The transfer is permanently recorded in the history, including the former owner's chosen status.

The former owner's status is an explicit part of the request. It is never assigned automatically.

### After a transfer

- A former owner who stays as a project manager or collaborator holds only that preset's capabilities. They do not keep any owner-only operation.
- A former owner who leaves is treated like a removed member (§4.7): they keep their own entries as read-only history, cannot add new ones, and their entries remain visible to the new owner.
- Amendments made by the former owner stay in place under their authorship. The new owner may withdraw them.
- Closed invoices stay as they were issued, attributed to the former owner.
- Offline devices learn of the transfer at their next sync. The hub evaluates each queued change under the ownership and permissions in force when it **receives** the change (§4.5).

Still to be designed (§22): what moves with the project (rates, clients, categories, shared tags, currency), and the rate for a former owner who stays as a member.

## 4.9 Authorization rules

- **Default deny.** Anything not explicitly granted is refused.
- **One decision everywhere.** The same evaluator answers for the interface, local changes, queued sync changes, hub routing and any future API.
- **No privilege escalation.** A person can grant only capabilities they hold and may delegate, and cannot assign a role above their own. A project manager cannot grant owner-only operations.
- **Permissions drive delivery.** What a person isn't entitled to is never sent to their devices (§13.1), not merely hidden by the interface.
- **Role changes** reach offline devices at the next sync. Data already cached is not recalled (§4.5).

Still to be designed (§22): override precedence, and whether project managers may grant the manager role.

---

# 5. Hub Accounts and Authentication

None of this applies in standalone mode. There is no sign-in, no account and no password locally.

## 5.1 Creating accounts

1. The hub operator creates an account (handle, display name, email) with the command line.
2. The system produces a one-time **invite link**. The operator copies it and sends it personally. The hub sends no email.
3. The invitee opens the link in a browser on a minimal hub page and registers a sign-in method.
4. The invite expires after a short period and can be reissued.

## 5.2 Sign-in

- **Passkeys** are the baseline for everyone. They are tied to the hub's web address, so sign-in happens on a minimal page served by the hub and opened in the system browser. The page then returns the user to the local app.
- The hub serves only: sign-in, invite acceptance and passkey registration, device pairing and approval, and recovery. It does not serve the application, entries, rates or administration.
- Google sign-in remains an optional later addition.
- Whether a password-based method is offered is open (§22).

## 5.3 Devices and sessions

- After sign-in, a device holds a long-lived credential in the operating system's secure storage and can work offline.
- Each device can be revoked. Disabling an account revokes all its devices.
- Sign-ins, failed sign-ins, device pairing and revocation are recorded in the hub's log.

## 5.4 Account states

- **Active** and **disabled**. A disabled account cannot sync. Its data remains.
- An account that owns data is never hard-deleted. An account with no data and no references may be removed by the operator.

## 5.5 Hub operator command line

- A command run on the hub's server, with no network-facing login.
- It can: create an account, update an account's email, issue a fresh invite link, disable and re-enable an account.
- Every use is recorded in the hub's log.
- This is also the recovery path for a person who has lost all sign-in methods.
- The operator necessarily has access to the server and therefore to the data it holds (§6).

---

# 6. Trust and Data Handling

## 6.1 Hub trust model

For the first milestone the hub is **trusted with plaintext**. Whoever can read the hub's disk, database, backups or logs can read the data. TLS protects data on the network. Access control protects it from other users. Neither protects it from the operator or from a breach.

This is acceptable when a person runs the hub for themselves and people they know. It becomes a real cost when others operate hubs on your behalf (Phase 3).

## 6.2 Data the hub holds

| Category | Contents | Sensitivity |
|---|---|---|
| Accounts | Handle, display name, email, language and timezone, passkey public keys, devices | Medium |
| Project structure | Client, project and category names, descriptions, shared tags, membership and capabilities | Medium to high |
| Time entries | Project, category, dates and times, minutes, timezone, tags, billable flag, versions, split links | Medium |
| Entry descriptions | Free text, which often contains confidential client detail | **Highest** |
| Rates, amendments, invoices | Hourly rates, owner adjustments and notes, invoice snapshots | High |
| Sync and audit metadata | Event IDs, authors, device IDs, timestamps, tombstones, access logs with IP addresses | Medium |

**Never sent:** private passkey keys, data that is not synced, and data the recipient isn't entitled to.

## 6.3 Required mitigations

- Full-disk encryption on the hub server, and encrypted backups
- Logs never contain descriptions or rates
- A documented rule for operator access
- A versioned protocol so field-level encryption can be added later, with entry descriptions as the first candidate

## 6.4 Federation trust (Phase 4)

Trust between hubs is **explicit and directional**: hub A trusting hub B does not mean B trusts A. Hub operators establish and control it. Once a hub operator has authorized federation under a policy, users can collaborate across those hubs without approval for each interaction. Hub trust enables collaboration but never replaces project invitations and permissions. Identity, discovery, transport, revocation and queued-change handling are not yet designed.

---

# 7. Domain Model

Field lists are logical. Storage is left to the implementation.

## 7.1 Who owns what

| Record | Authority |
|---|---|
| Time entry, entry versions, personal tags, contributor invoice | The **author** (the member who logged the time) |
| Project, client, category, shared tag, membership, rate rule, amendment, client invoice | The **project owner**, or a delegate holding the capability |
| Account, profile, preferences | The **user** |

A record is changed only by its authority. Others record their own view in separate records.

## 7.2 Client

`id`, `ownerId`, `name`, `status` (active | archived). Contact fields are deferred. One client has many projects.

## 7.3 Project

`id`, `ownerId`, `clientId`, `name`, `description`, `status` (active | archived), `billableDefault`.

Derived, not stored: first and latest entry date, total time.

## 7.4 Category

`id`, `ownerId`, `name`, `status`. A category describes a type of work and carries no rate. Categories belong to the owner and are enabled per project. Members may use the categories enabled on their projects.

## 7.5 Tag

`id`, `name`, `kind` (shared | personal), `ownerId` (the owner for shared tags, the user for personal tags), `status`.

- **Shared tags** are managed by the project owner and available to members of their projects.
- **Personal tags** are created by anyone by typing a new tag on an entry. They are suggested only to their creator.
- A tag name is 1–40 characters, trimmed, unique ignoring case within the shared set and within each person's personal set. A name matching a shared tag uses the shared tag.
- An entry may carry up to 10 tags.
- The owner can promote a personal tag to a shared tag.
- A person can rename or archive their own personal tags, and delete them when unused.

## 7.6 Membership

`projectId`, `userId`, `preset`, capability overrides, `status` (invited | active | removed), invited-by, timestamps.

## 7.7 Time entry

| Field | Description |
|---|---|
| `id` | Identifier (§3.5) |
| `authorId` | The author |
| `projectId`, `categoryId` | Project and category |
| `kind` | interval, duration, or running |
| `workDate` | Calendar date in the entry's timezone |
| `timezone` | The author's timezone when the entry was made |
| `startAt`, `endAt` | Absolute instants (interval and running) |
| `minutes` | Whole minutes. Derived from the instants for intervals; stored for duration-only entries |
| `description` | Required (at stop for a timer) |
| `billable` | Defaults from the project, overridable |
| `tags` | Zero to ten |
| `startOffset`, `endOffset` | UTC offsets (minutes) in effect at the start and end instants, as the author's app recorded them |
| `tzdataVersion` | Version of the timezone database the author's app used. Informational only |
| `splitGroupId` | Shared by linked pieces of a midnight split. Each piece also records the original span (`spanStartAt`, `spanEndAt`) and its place in the group (`pieceIndex`, `pieceCount`) |
| `version` | Increases with every edit (§11.1) |
| `deleted` | Deletion marker (tombstone) |
| `contributorInvoiceId`, `clientInvoiceId` | Invoice locks (§12) |
| `source`, `externalRef` | Provenance and an external tool's identifier (reserved for §20) |
| `createdAt`, `updatedAt` | Device time and hub-received time |

## 7.8 Amendment

See §11.2.

## 7.9 Rate rule

See §10.

## 7.10 Invoice

See §12.

## 7.11 Change history

Every create, edit, archive, delete, lock and permission change is kept as an append-only history identifying the author, the device, the device time and the hub-received time, with previous and new values. Authors see the history of their own records. Owners see the history of entries on their projects.

## 7.12 Settings

Per installation: language, timezone (IANA), date format, clock format (24h | 12h), theme, entry window (days). Per owner: currency (default EUR).

---

# 8. Time Recording

## 8.1 Three ways to record

**A. Start and end.** Enter start and end times (and optionally a date, default today). Duration is calculated.

**B. Duration only.** Enter a duration as `hh:mm` and a date (default today). Start and end stay empty and show "–" in the From and To columns.

**C. Live timer.** See §9.

All three produce the same kind of entry and can be edited later.

## 8.2 Fields

Required: project, category, description, and either (start and end) or a duration. Optional: date (default today), tags, billable flag (defaults from the project).

## 8.3 Dates and timezones

- Entries default to today in the user's timezone and may be back-dated. Existing entries may be modified until locked (§12).
- Entries dated or ending in the future are rejected.
- `workDate` is a plain calendar date computed in the entry's stored timezone. Changing the timezone preference affects only later entries.
- An ambiguous wall-clock time (the repeated hour when daylight-saving time ends) uses the first occurrence.
- Times are recorded as instants together with their zone and offsets, so their meaning never depends on the viewer's or the hub's timezone data (§8.5).
- Device clocks may be wrong. The hub also records when it received an entry. The future-entry check uses the device clock locally and the hub's received time with a tolerance on arrival.

## 8.4 Validation

- End must be after start. Zero and negative durations are rejected.
- An interval may not exceed 24 hours. A duration-only entry must be between one minute and 24 hours.
- The project and category must be available to the author and not archived.
- **Overlaps:** if an interval overlaps another interval of the same person, the app warns and allows the save. This covers all of that person's projects. Duration-only entries never take part. Different people overlapping is normal.
- The hub re-validates every rule on arrival (§13.6).

## 8.5 Midnight splitting

When an interval crosses midnight in the entry's timezone, it is split automatically into one entry per calendar day:

- 23:00 to 02:00 becomes 23:00–24:00 on the first day and 00:00–02:00 on the next.
- The pieces share a `splitGroupId` and show a visible link.
- Description, category, project, tags and billable flag are the same on each piece.
- Editing treats the group as one span: change the start of the first piece or the end of the last, and the pieces are re-split. If the crossing disappears, they merge.
- Deleting one piece deletes the group, after a confirmation that says so.
- If one piece is locked by an invoice, the locked piece is frozen and the other can be edited, but not in a way that moves the midnight boundary (§12.8).

### The client performs the split

- The author's app calculates the split **when the time is recorded**, using the entry's timezone, and records the result. The hub stores and delivers it. It never recalculates it, with its own timezone data or any other.
- The recorded result is the authoritative interpretation. It stays unchanged unless the author edits the entry's timing.
- Each piece records its start and end instants (UTC), the entry's IANA timezone, the UTC offset in force at its start and end, its local calendar date, its position in the group, and the original span of the whole group. The timezone database version used is recorded for diagnosis only.
- **Elapsed time comes from the instants.** A split across a daylight-saving change has the real duration, not the difference between wall-clock times.
- If a zone has no local midnight on a given day (a transition at midnight), the boundary is the first instant of the new calendar date.
- Editing only non-timing fields (description, category, tags, billable flag) never recalculates a split. Editing the timing recalculates it on the editing device, using the entry's recorded timezone.
- The pieces of a group are created, changed, delivered and rejected **together**.
- An entry has exactly one timezone. For a timer it is the zone in force when the timer started.

### What the hub checks

The hub checks consistency from the recorded values alone, with no timezone database:

- the pieces are contiguous and cover the original span exactly, with no gap or overlap
- the elapsed minutes of the pieces add up to the elapsed minutes of the span
- each piece's local date equals the date of its start instant shifted by its recorded offset, and every internal boundary is local midnight under the recorded offset
- offsets lie in a plausible range (−12:00 to +14:00)
- the group is complete

An inconsistent split is rejected like any invalid change (§13.6). The hub may compare a recorded offset with its own timezone data and warn the owner of a discrepancy. A warning is never a reason to reject or alter the record.

### Recording versus reporting

Recording fixes which calendar day an entry belongs to and how long it lasted. Reporting never changes that. Someone who records time in Costa Rica for a project owned in Germany keeps their Costa Rican dates and totals when the owner views them in Germany.

Times are shown in the zone the author recorded them in, labelled when that differs from the viewer's. A viewer may choose to see times in their own zone, but the dates and totals used for accounting do not move.

### Travelling across zones

A person may record in one zone in the morning and another in the evening. Hours never depend on the zone, because a duration is the difference between two instants. A zone decides only which calendar date an entry belongs to and where a midnight split falls.

- Each entry has its own zone. A day's total is the sum of its entries by their own local dates.
- Overlap between entries is judged on the instants, never on local clock times.
- Reports and invoice periods use each entry's own local date, so dates never move (see above).
- There is no setting that attributes every day to a fixed "home" zone, because changing it would move dates and break locks. A display-only option is possible later and would never change stored dates.
- Editing an entry's times interprets them in the entry's own zone. Changing the zone is a separate, explicit action.

**Proposed (open items #37 and #38).** The zone field defaults to the device's current zone, and the entry shows its zone beside its times whenever it differs from the previous entry or the device, with one action to change it, because work entered later may belong to an earlier zone. A running timer that crosses a zone change is split silently at that instant into linked pieces, each with its own zone and offsets. Each piece then records why it begins where it does, `midnight` or `zone_change`. The hub checks a midnight boundary as described above, and a zone-change boundary by requiring contiguous pieces at the same instant with different zones.

## 8.6 Entry form

- A compact single row on wide screens, wrapping on narrow ones.
- **Enter** in the description submits, **Shift+Enter** inserts a new line.
- After saving, the form clears, the table and daily totals update immediately, and focus returns to the first field.
- **Smart defaults:** the start time is proposed as the end of the previous entry that day, and the category as the last one used on the project.

## 8.7 Feedback

Clear feedback on success, validation failure, rejection on sync and version conflicts. A failed save never discards what the user typed.

---

# 9. Live Timer

- A person has at most **one** running timer. The timer lives on the device and syncs with the person's other devices.
- Start by choosing project and category. Description and tags are optional at this point.
- A persistent indicator with elapsed time is visible in the header.
- Stop: the description is confirmed (required). The entry becomes an ordinary interval and is split if it crosses midnight.
- Start and stop times are captured to the minute.
- A start control may appear on each dashboard project card.
- When two offline devices each start a timer, the rule "one at a time" is reconciled at sync, and the person is prompted.
- **Stale timer.** A timer left running from a previous day prompts the person at next launch to enter the real stop time and description, or discard it. This is the only notification in the first milestone.

---

# 10. Rates and Valuation

## 10.1 Concept

A **rate** is the hourly value of one member's time, defined by the project owner. What the owner's client is charged is not tracked.

- The owner defines each member's rates. A member sees only their own.
- A member who works for several owners has separate rate sets per owner.
- Rates belong to the owner's namespace and use the owner's currency (§7.12).

## 10.2 Rate rules

A rate rule contains:

- `ownerId` and `userId` (the member)
- optional `clientId`, `projectId`, `categoryId`
- `amount` (hourly)
- `effectiveFrom` (a date; the first rule for a member applies "from the beginning")
- who created it and when

Rules are **immutable**. To change a rate, add a rule with a later `effectiveFrom`. The previous rule ends where the next begins.

Every member has a **base rate**: a rule with only the member set. It is required when a collaborator is first added to an owner's projects. An owner's own time may have no rate; such entries show "no rate", never zero. There is no system-wide default.

## 10.3 Resolving the rate for an entry

For an entry with a member, client (via the project), project, category and `workDate`:

1. For each distinct scope (the combination of fields that are set), take the rule with the latest `effectiveFrom` on or before the `workDate`.
2. Keep the scopes whose set fields all match the entry.
3. If one scope is **more specific** than all others, use it. Scope A is more specific than B when everything A matches is also matched by B. A project rule is more specific than its client's rule, and a rule with more fields is more specific than one with a subset of them.
4. If no scope is more specific than all others, there is a **conflict** (§10.5).
5. The base rate is the fallback.

Rates are looked up by the entry's work date, not when it was entered.

## 10.4 Entry value

`value = minutes × hourlyRate / 60`, rounded half-up to the smallest currency unit **once per entry**. Totals are the sum of entry values. Billable status does not change value; it marks whether hours are billable to the client.

Values of invoiced entries come from the invoice snapshot (§12.4), not from current rates.

## 10.5 Conflicts

A conflict exists when matching rules overlap and neither is more specific. Example: "Development" at €120 and "client Acme" at €110 both match Doug logging Development for Acme.

- Conflicts are an edge case, never resolved silently or by a hidden priority order.
- When the owner saves a rule that would create one, the app says so and asks for the rate of the combined case (Doug + Acme + Development), which resolves it.
- If an entry is saved while a conflict exists (for example after a back-dated rule), it is flagged **rate conflict**, excluded from value totals, and the owner sees an alert until it is resolved.
- Every resolution is recorded in the history.

## 10.6 Rate table

The owner has a table per member showing the current rate, all overrides, and an expandable history with effective dates.

## 10.7 Worked examples

Base rates set by owner Baden: Doug €100/h, Steffen €70/h.

| Case | Result |
|---|---|
| Doug, 90 min, base rate | 90 × 100 / 60 = **€150.00** |
| Steffen, 20 min | 20 × 70 / 60 = 23.333… → **€23.33** |
| Steffen, 25 min | 25 × 70 / 60 = 29.1666… → **€29.17** |
| Doug base €100 until 2027-01-01, then €110. Entry on 2026-12-31 | **€100** |
| Same, entry on 2027-01-01 | **€110** |
| Doug + Development = €120, Doug + project Acme/Website = €115. Doug logs Development on Acme/Website | **Conflict** |
| Add Doug + Acme/Website + Development = €130, then the same entry | **€130** |
| Doug + client Acme = €110, Doug + project Acme/Website = €115. Doug logs on Acme/Website | **€115** (project is more specific than client) |

---

# 11. Entry Versions and Owner Amendments

## 11.1 Entry versions

Each edit of an entry creates a new **version**. Earlier versions are kept in the history. Authors see their own history, and owners see the history of entries on their projects. This is what lets an invoice pin exactly what was billed (§12).

## 11.2 Amendments

An owner (or a person with `amend`) may need a different figure from what the collaborator recorded, for example 3:00 instead of 3:30. The owner **cannot rewrite the entry**. Instead the owner records an **amendment**:

- `entryId` and the entry version it refers to
- `adjustedMinutes` and/or a `billable` override
- a required `reason`
- author, timestamps, and a status (active | withdrawn)

The **effective** figure for the owner's reports is the original entry combined with its latest active amendment. Amendments are append-only. A new one supersedes the previous.

## 11.3 Rules

- An amendment affects only the owner's accounting. It never changes the author's entry, the author's invoice, or the author's own totals.
- Authors can see that an amendment exists, with the adjusted figure and reason, read-only (§22).
- An amendment to an entry already on a closed client invoice does not alter that invoice. It takes effect as an adjustment in the next open period.
- If the author edits an entry after an amendment, the amendment is shown as **refers to an earlier version**, and the owner decides whether to keep or withdraw it.
- Provenance is always preserved: the original assertion, each edit, and each amendment.

---

# 12. Invoicing and Locks

TimeTrack does not produce invoice documents. It records that a set of hours has been invoiced, and locks it.

## 12.1 Two kinds of invoice

| | Contributor invoice | Client invoice |
|---|---|---|
| Issued by | The member who logged the time | The project owner (or a holder of `invoices.close`) |
| Covers | That member's own entries only | All members' entries on the chosen projects |
| Recipient | One project owner | One client |
| Multiple projects | Yes, if they share the same owner | Yes, if they share the same client |
| Period | Any date range, a calendar month by default | Same |
| Value | Minutes × the rate the owner set for that member | Minutes and values with owner amendments applied |
| Reopened by | The issuer, with a reason | The owner (or holder of `invoices.reopen`), with a reason |

An entry can be in at most one invoice of each kind.

## 12.2 Closing

Closing an invoice:

1. Selects the entries in the chosen projects and period that are not yet in an invoice of that kind.
2. Pins each at its current version, with its minutes, rate and value (the snapshot, §12.4).
3. Marks them invoiced and locks them.
4. Records an optional external invoice reference, who closed it, and when.

Entries dated in the period but not yet synced to the hub, or logged afterwards, are **not** included. They stay uninvoiced and go on the next invoice. A client invoice treats entries with an open conflict (§13.3) the same way; a contributor invoice is refused until the member resolves them.

## 12.3 Lock effects

- An invoiced entry (either kind) can't be edited or deleted, by anyone, including the owner.
- A contributor invoice locks the member's entries.
- A client invoice locks all included entries, including the owner's own, and the members' entries too, whether or not they had issued a contributor invoice.
- If a client invoice is closed before a member has issued theirs, that member **can still issue a contributor invoice** for those entries. Their lock is independent.
- New entries for a closed period are allowed. They are uninvoiced and appear on the next invoice of both kinds.
- If the owner is also the author, only the client invoice applies.

## 12.4 Snapshot

At close, each pinned line stores the entry version, minutes, rate and value. After close:

- reports for invoiced entries use the snapshot
- a later rate change, including a back-dated rule, does not alter a closed invoice
- an amendment to a pinned entry becomes an adjustment in the next open period

## 12.5 Owner workflow

When closing a client invoice, the dialog shows each member's status (for example "Doug: invoiced. Steffen: not yet") and their last sync time, because the hub cannot see changes still on offline devices. The owner may close anyway. An optional filter includes only entries that members have already invoiced.

## 12.6 Reopening

- A contributor invoice is reopened by its issuer, a client invoice by the owner or holder of `invoices.reopen`.
- A reason is mandatory. It is recorded and visible to the other party.
- A member **cannot** reopen a contributor invoice while a client invoice includes those entries. The owner must reopen theirs first.
- Reopening unlocks the pinned entries; closing again produces a new snapshot.
- There is no "accepted" state in the first milestone (§22).

## 12.7 Ordering and offline

The hub orders locks and edits by the time it **received** them, never by device timestamps. An edit made offline before a close, but received after, is **rejected**. The device shows the entry as invoiced with the reason, and keeps the rejected change as a visible draft so the person can ask the owner for a correction.

## 12.8 Edge cases

| Case | Rule |
|---|---|
| Edit or delete an invoiced entry | Rejected for everyone |
| Split entry across a locked boundary | The locked piece is frozen. The other can be edited, but the midnight boundary cannot move |
| Hard delete of a project or client with invoices | Blocked. Archive only |
| What collaborators see | "Locked: invoiced" and, for client invoices, "billed to the client". The client invoice's reference is hidden from members |
| Standalone | The same close action exists as a guard against yourself |

---

# 13. Sync and Collaboration

This section states requirements for synchronization. The mechanism is open.

## 13.1 What each party receives

| Party | Receives |
|---|---|
| A person's other devices | Everything that person is entitled to |
| A member | Their own entries and rates; the project's reference data (project, client name, enabled categories, shared tags, members' display names); their invoices and lock states |
| An owner or holder of `entries.viewAll` | All entries on their projects, amendments, invoices, and (with `rates.viewAll`) rates |

The hub filters per recipient. A device never receives data its person isn't entitled to.

## 13.2 Guarantees

- **Eventual convergence.** Participants end with the same data for everything they are entitled to, after reconnecting.
- **Queued delivery.** The hub retains changes for disconnected participants and delivers them on reconnection.
- **Idempotent.** Delivering or submitting the same change twice has the same effect as once.
- **No silent loss.** A change that cannot be applied is surfaced to the person, never dropped.
- **No silent overwrite.** Competing edits are never settled by last-write-wins. A person chooses (§13.3).
- **Offline use.** Every local operation works offline and is queued.

## 13.3 One writer per record, and conflicts

By construction, nearly all records have one author, so edit conflicts between different people do not arise. Two cases remain.

**Competing edits to one record.** The same person edits a record on two devices, or the owner and a delegate edit the same owner-authority record (for example project settings) at once.

**Authority across records.** A permission change, a lock, an ownership transfer or a membership removal races an offline edit. This is not settled by a choice. The hub decides by the order it received them (§13.6).

### Detection

Every change names the version it was based on. The hub accepts it when that is still the current version. If the base is older, the hub compares what each side changed:

- **Different fields changed:** the changes merge automatically.
- **The same field changed to different values:** this is a **conflict**.

For an entry, the fields that conflict together are: timing (date, start, end, duration), category, description, tags, billable flag. A split group is treated as one unit.

### Presentation and resolution

- The first accepted version stays current and is what everyone else sees. The later change is held on the device where it was made and marked as a conflict. The entry is flagged.
- The person sees both versions side by side with their origins (device and time) and chooses: keep mine, take the other, or combine them by editing.
- The choice becomes a new version. The history records both competing versions, who chose and when.
- Only the person with authority over the record resolves its conflict: the author for an entry, the owner (or the delegate holding the capability) for project records. Nobody resolves someone else's conflict.
- An entry with an unresolved conflict cannot be marked as invoiced. A contributor invoice is refused while the member's own entries in it have an open conflict, and lists them. A client invoice excludes such entries, lists them, and they go on the next invoice, so one person's unresolved edit never blocks the owner (§12.2).

This is the product behavior. The replication mechanism that supports it is open (§22).

## 13.4 Deletions

Deletion is recorded as a marker (a tombstone) that syncs like any change, so deleted records disappear everywhere and cannot reappear.

## 13.5 Joining a hub

### First run

The first launch of an installation asks how it will be used:

- **Stand-alone.** A local identity is created. There is no account and no network use. The person can connect to a hub later from Settings.
- **Part of a network** (sync between a person's own devices, collaboration with others, or both; both need a hub). The person either **joins an existing hub** with an invite link or a hub address, or **creates a hub** (see below).

Two installations cannot sync with each other directly. They need a hub.

### Joining a hub

The app checks the hub's protocol version and records its permanent ID next to its DNS name, and warns if the name later points at a different hub. The person signs in on the hub's page in the system browser. A new account is made from an invite. A fresh installation pulls everything the person is entitled to. A new device for an existing account is just another device.

An installation that already holds stand-alone data asks which projects to bring to the hub. The first installation to join claims its local identity as the account. A later installation signing in to the same account has its local identity recorded as an **alias** of the account, and the projects brought over are attributed to the account, with the original identity kept in provenance and an audited "install linked" event. Projects not chosen stay local and are never uploaded. Import is all-or-nothing per project, after a check.

Projects and clients from two installations stay separate even when their names match. There is no project merge in the first milestone.

A person can disconnect and return to stand-alone, keeping their data. The hub keeps its copy.

### Creating a hub

Creating a hub is an operator task, not a function of the desktop app. The app explains what is needed and then continues as for joining a hub once the operator has an address and a first invite. The hub's sign-in page needs a stable name and a valid certificate, because passkeys are bound to the web origin. The first account is created with the operator command line (§5.5). The options for where a hub runs are discussed in [docs/architecture.md](docs/architecture.md) (§5.9).

## 13.6 Hub validation

The hub re-validates everything on arrival: authorship, membership and capabilities, project and category availability, locks, time rules, and the future-entry check against received time. Midnight splits are checked for structural consistency only (§8.5). The hub never recalculates them and needs no timezone database. A rejected change returns a reason, and the device shows it in a **needs-attention** list with the person's draft preserved.

## 13.7 New and recovered devices

A new or recovered device pulls everything its person is entitled to. A lost device is revoked (§13.8) and its person starts afresh from the hub. Unsynced changes on a lost device are gone.

## 13.8 Revocation

Revoking a device, removing a member, or disabling an account stops future sync. It cannot remove data already on a device (§4.5).

## 13.9 Status

The app shows: connected or offline, last synced, number of pending changes, and the needs-attention list.

## 13.10 Protocol versions

The sync protocol is versioned. A hub may refuse versions below a minimum and tell the app to update. A local database is migrated safely when the app updates (§16).

---

# 14. Screens

## 14.1 Dashboard

- A grid of cards for the user's active projects, including projects they own and those they are members of, with the owner shown for shared ones.
- Card: project name, client name, description (truncated), a timer start control.
- Keyboard and pointer accessible. Empty state offers to create a first project.

## 14.2 Project time page

1. **New entry form** (§8.6), with a mode switch for start/end, duration or timer.
2. **Entries**, grouped by date, newest first, one table per date. The group header shows the date and day total in `hh:mm`.

Only the last `entryWindow` days load by default, with a control for older entries. Viewers with `entries.viewAll` see all members' entries with a user column and filter.

Columns: From, To, Duration (`hh:mm`, read-only), Category, Description, Tags, Billable, and Value where the viewer may see it. Rows show a lock for invoiced entries, a link for split pieces, and a marker for amendments. Empty state: "No time entries have been recorded for this project."

## 14.3 Row-level editing

- Activating a cell (click or double-click) makes the **whole row** editable, with focus in the chosen cell.
- **Enter** saves, **Esc** cancels, with visible Save and Cancel.
- The row is validated and saved as one change, producing one new version.
- **Tab** moves between fields in the row. Leaving the row with unsaved changes asks to keep or discard.
- Editing start or end recalculates duration, splits or merges across midnight, and updates totals.
- Touch devices get an explicit Edit button.
- A failed save keeps the row open with the edited values and lets the person retry or cancel.
- Locked entries cannot enter edit mode and explain why.
- Editable fields: date, start, end (or duration), category, description, tags, billable.

## 14.4 Search

- Searches **everything the user may see**, not just the window. In the first milestone this works offline.
- Free text over descriptions, with filters for client, project, category, tag, billable flag, date range, invoice state and (where permitted) person.
- Results show total time and, where permitted, total value for the filtered set.

## 14.5 Project settings

Members and their roles and capabilities, invitations, categories enabled, shared tags, rate tables (§10.6), rate conflicts, archive and delete.

## 14.6 Invoices

A list of contributor and client invoices with state, period, projects, totals and reference; the close dialog (§12.5); reopening with a reason.

## 14.7 Amendments

Owners can amend an entry from the project page, with a required reason. Amended entries show original and effective figures.

## 14.8 Sync and account

Hub connection (connect, sign in, disconnect), account details, devices with revoke, sync status, and the needs-attention list. Conflicts open a side-by-side comparison where the person chooses a version (§13.3).

## 14.9 Navigation

Persistent navigation: Dashboard, Projects, Search, Invoices, and Settings. A user menu with preferences, theme switch and, when connected, account and sync status. The current location is identifiable, and menus are keyboard accessible.

---

# 15. Archive and Delete

| Item | Archive | Hard delete |
|---|---|---|
| Client | Yes (default). Hides it and its projects; history remains. | Owner only, danger zone, with counts and an explicit confirmation. Blocked if any invoice exists or other people's entries exist. |
| Project | Yes (default). | Same. |
| Category | Yes. Existing entries keep it. | Never while referenced by any entry or rate rule. |
| Tag | Yes. Archived tags are no longer suggested. | Never while in use. A personal tag by its owner, or a shared tag by the project owner, when unused. |
| Entry | n/a | By its author, with confirmation, unless locked. Recorded as a tombstone with a snapshot in the history. |
| Member | Remove (stops future sync) | Not applicable |
| Hub account | Disable | Only if it owns no data and has no references |

Archived items can be restored by the owner. Deletions touching several records are atomic. Hard delete is blocked when another person's entries would be destroyed; archive instead. Statutory retention of billing records may restrict deletion (§22).

---

# 16. Backup, Restore and Standalone Guarantees

## 16.1 Standalone guarantees

- No network calls, no telemetry and no remote assets unless the person connects to a hub. Checking for updates, if offered, is opt-in.
- No account or sign-in. Data is stored locally in a form that is easy to copy.

## 16.2 Backup and restore

Because standalone data exists nowhere else, the first milestone includes:

- A manual **backup** to a single file the person can store anywhere
- A **restore** from a backup file, with confirmation
- A reminder when no backup has been made for a set time

On a hub, the operator is responsible for backups (§19.5).

## 16.3 Safe upgrades

The local database is versioned. The app migrates it on update and keeps the previous copy until the migration succeeds. A failed migration never leaves unusable data.

---

# 17. Preferences, Language and Appearance

## 17.1 Preferences

- Language (default English)
- Timezone (IANA), date format (default `DD.MM.YYYY`), clock format (default 24-hour)
- Theme (light, dark, or system)
- Entry window (default 60 days)

Duration is always `hh:mm`. Dates and times are shown in the viewer's formats and timezone. Language and date format are independent.

## 17.2 Language and translation

- All code, comments, identifiers and developer documentation are in English.
- Every piece of user-visible text lives in translation catalogs separate from code. No screen contains hard-coded text. Adding a language is adding a catalog.
- A missing translation falls back to English. A raw key is never shown.
- Messages with values or counts use parameters and plural rules. Sentences are not assembled from fragments.
- The hub reports errors as stable codes plus parameters, which the app translates.
- The change history stores codes and values and is displayed in the viewer's language.
- User-entered data (names, descriptions) is not translated.
- Layout tolerates longer text and uses direction-neutral styling so right-to-left languages can be added.

## 17.3 Appearance

Light and dark themes with a switch. Centralized theme tokens, no hard-coded colors.

## 17.4 Accessible and responsive

- Desktop first, usable on smaller screens.
- Keyboard navigation, visible focus, labelled inputs, accessible errors, sufficient contrast, semantic tables, accessible dialogs.
- Every pointer-only interaction has a keyboard alternative.

---

# 18. Calculations and Test Vectors

## 18.1 Rules

- Duration of an interval = whole minutes between `startAt` and `endAt` (real elapsed time).
- Display `hh:mm`, hours not capped at 24 (for example `126:30`).
- Daily total = sum of that date's minutes for the viewed person or the viewer's entitled set.
- Totals are computed from minutes, never from displayed values.
- Values follow §10.4. Totals in different currencies are never added.

## 18.2 Time examples

| Case | Result |
|---|---|
| 09:15 to 12:45 | 210 min = **3:30** |
| 09:00–11:30, 11:45–13:00, 14:00–17:00 | 150 + 75 + 180 = 405 min = **6:45** |
| Edit 09:00–11:00 to 09:00–11:15 | 2:00 → **2:15**; day total +0:15 |
| 23:00 to 02:00, Europe/Berlin, from 2026-11-02 | Two entries: 11-02 23:00–24:00 (1:00) and 11-03 00:00–02:00 (2:00); one group; **3:00** |
| Edit that span's end to 23:30 | Merges into one entry on 11-02, **0:30** |
| 01:30 to 03:30 on 2026-03-29, Europe/Berlin (02:00 jumps to 03:00) | Real elapsed **1:00** |
| 23:00 on 2026-10-24 to 04:00 on 2026-10-25, Europe/Berlin (clocks go back at 03:00) | Split at midnight into **1:00** (24 Oct) and **5:00** (25 Oct). Real elapsed **6:00**, not the 5:00 wall-clock difference |
| A Costa Rica author records 22:00–23:30 on 2 Nov; the owner views the report in Germany | The entry stays on **2 Nov**, **1:30**. Times are labelled with the author's zone |
| A split recorded by a client whose timezone database is newer than the hub's | Hub accepts it unchanged and does not recalculate |
| A split with a gap, an overlap, a missing piece, or minutes that don't add up to the span | Hub **rejects** the whole group |
| Edit only the description of a split entry | Split unchanged, not recalculated |
| Duration entry 1:30 | No start or end, **1:30** |
| End before start, start equals end, over 24 hours, in the future | Rejected |
| Overlap with own entry on any project | Warning; save allowed |
| Overlap with another person's entry | No warning |

## 18.3 Invoice and sync examples

| Case | Result |
|---|---|
| Doug logs 2,550 minutes in October and closes a contributor invoice (rate €100) | **42:30**, 2,550 × 100 / 60 = **€4,250.00** |
| Doug adds a 1:30 entry dated 30 Oct on 5 Nov | Uninvoiced. October invoice still **42:30 / €4,250.00**. Billed next time |
| Baden adds a Doug rate of €110 effective 2026-10-01 after October closed | Closed invoice stays **€4,250.00**. The late entry later values at 90 × 110 / 60 = **€165.00** |
| Baden closes a client invoice before Steffen has invoiced | Steffen's entries are locked from editing. He can still issue his contributor invoice. New entries are uninvoiced |
| Doug edits an October entry offline on 2 Nov; Baden closes October on 3 Nov; Doug syncs on 4 Nov | Edit **rejected**, entry shows as invoiced, the draft is kept |
| Doug tries to reopen his invoice while a client invoice includes it | **Refused** until Baden reopens his |
| Baden amends Doug's 3:30 entry to 3:00 before closing | Client invoice uses **3:00**. Doug's entry and invoice stay **3:30** |
| Entry 31 Oct 23:00 to 1 Nov 02:00 with October closed | Oct piece locked. Nov piece editable, but its start can't move |
| Same person edits the description of one entry on two offline devices | The first change to reach the hub stays current. The other is held as a **conflict**. The person chooses a version, and the history records both |
| Same person edits the description on one device and the category on another | Both changes **merge automatically** |
| Owner closes an invoice while an entry has an unresolved conflict | Entry pinned at the accepted version. The held change is rejected |

---

# 19. Non-Functional Requirements

## 19.1 Security

- Authenticated, authorized access to all synced data, enforced at the hub.
- Secure device credentials in the operating system's secure storage.
- Encrypted transport. Protection against injection and cross-site attacks on the hub's pages.
- Rate limiting and lockout on repeated failed sign-ins.
- No plain-text secrets. Secrets kept out of source control.
- Hub exposure should be minimized, for example by running it behind a private network.

## 19.2 Reliability and integrity

- Referential integrity between related records.
- Multi-record operations are atomic.
- Failed saves and rejected changes never silently lose user input.
- Idempotent sync (§13.2).
- The hub validates every rule, except that it does not reinterpret time zones: the author's app records that interpretation and the hub checks it for consistency.

## 19.3 Performance

- Local operations are immediate, regardless of connectivity.
- Default views are bounded by the entry window; search and long histories stay responsive.

## 19.4 Maintainability

- Business rules (time, splitting, rates, invoice locking) exist in **one** implementation shared by the app and the hub.
- Validation is centralized and reused by create and edit.
- Table columns and form fields are declared in configuration.
- The interface and any future API use the same application service layer.
- All code and comments are in English.

## 19.5 Operations (hub)

- Backups with a tested restore, disk encryption, HTTPS, monitoring and error reporting.
- A documented procedure for the operator command line.
- Logs exclude descriptions and rates.

## 19.6 Testing

Automated tests are required for:

- Duration, totals, `hh:mm` and value calculation, using the vectors in §10.7 and §18
- Midnight splitting, merging, editing, and daylight-saving days
- Recorded splits: the hub's consistency checks, rejection of gaps, overlaps and missing pieces, acceptance across differing timezone databases, display of times in the author's zone without moving dates
- Rate resolution, effective dates, specificity and conflict detection
- Timezone handling, including a changed preference
- All validation rules, locally and at the hub
- Capability enforcement at the hub: visibility, filtering, removal, revocation
- Amendments never changing the author's data
- Invoice closing, snapshots, locks, reopening, late entries and the cross-lock rule
- Offline behavior: queueing, idempotent delivery, rejection of stale edits, ordering by hub receipt
- Conflict handling: detection by base version, automatic merge of non-overlapping edits, side-by-side resolution of overlapping ones, the choice recorded in the history
- Authorization: default deny, no escalation, owner-only operations never delegable, atomic ownership transfer, one evaluator for all paths
- Tombstones, joining and leaving a hub, new and revoked devices
- Backup, restore and database migration, including a failed migration
- Authentication flows: invite, passkey, device pairing, revocation, operator recovery
- Timer behavior, stale timers, and the two-device case
- Translation: every key exists in English, fallbacks, plurals and parameters
- Accessibility of keyboard flows, and both themes
- Standalone makes no network calls

---

# 20. Planned API (designed for now, built later)

## 20.1 Intent

A developer who has built their own tool should be able to connect to a hub with an API key to read the reference data they are allowed to use and submit time against those projects.

## 20.2 Constraints on the first milestone

- One application service layer is used by the app, the hub's sync, and any future API.
- Stable, opaque identifiers; machine-readable error codes; unambiguous types (instants with offsets, integer minutes, money in the smallest unit plus currency).
- Versioned records; lists that are paginated and filterable.
- Writes are idempotent. An `externalRef` repeated by the same person returns the existing entry.
- Entries record `source` and optional `externalRef`.

## 20.3 API keys

- A key belongs to one account and **acts as that person**, with exactly their permissions.
- Scopes: read-only, or read plus write of own time entries.
- Shown once, stored only as a hash, with label, optional expiry, last-used time, and revocable at any time.
- A key cannot manage accounts, devices, members, rates or invoices.
- Writes made with a key are recorded in the history with the key.

## 20.4 Intended first release

Read: own profile, projects, clients, categories, tags, own entries. Write: create, update and delete own entries (subject to locks). Rates, invoices and the timer are not exposed at first.

## 20.5 Contract

Versioned, published as a machine-readable description with examples, with deprecations announced.

---

# 21. Later Phases

## 21.1 Phase 3: independent hubs

Multiple operators run hubs. A person belongs to one hub per account. Required design before this phase: hub discovery, how an installation connects to its hub, and how data moves if a hub moves name (§3.2).

## 21.2 Phase 4: federation

Trusted hubs exchange data (§6.4). Required design: hub identity and trust establishment, cross-hub user identity, discovery, transport security, event routing, permission enforcement across hubs, and what happens to queued changes when trust is revoked.

## 21.3 Other future work

- Reporting module: totals by client, project, category, person and period; billable summaries; exports
- General data export and import
- An "accepted" state for contributor invoices
- Cost versus client billing rates
- Project and task budgets
- Expense tracking
- Email notifications
- Mobile apps and a web client
- Windows and Linux builds
- Field-level or end-to-end encryption
- Additional sign-in methods
- Additional interface languages
- The public API (§20)

---

# 22. Open Items

Each has a working default that the specification above already assumes.

| # | Item | Working default |
|---|---|---|
| 1 | Interface framework, operating-system integration, database, transport and deployment | Open. The direction is a shared Rust domain core (`docs/architecture.md`) |
| 2 | Replication mechanism behind §13 | Open; requirements are in §13 |
| 3 | Sign-in on the hub: browser passkeys or device-held keys | Browser passkeys |
| 4 | Whether a password method is offered (v1 allowed it for admins) | No; passkeys only, recovery through the operator |
| 5 | Google sign-in | Later, optional |
| 6 | Categories: per owner with per-project enablement, or per project | Per owner, enabled per project |
| 7 | Restricting which categories an individual member may use | Not in the first milestone |
| 8 | Visibility of amendments to the author | Visible, read-only |
| 9 | Whether project managers see rates | No, unless granted |
| 10 | Currency per owner versus per project or client | Per owner |
| 11 | Who may create hub accounts | Only the operator |
| 12 | Whether ordinary users may create projects on a hub | Yes, subject to hub policy |
| 13 | Accepted state for contributor invoices | Later |
| 14 | Hard delete of a project that has other people's entries | Blocked; archive instead |
| 15 | Audit and history retention and personal data (IP addresses, names) | Keep until a policy is set |
| 16 | Statutory retention of billing records in Germany | Confirm with an accountant; hard delete is owner-only and blocked when invoices exist |
| 17 | Maximum back-dating | Unlimited |
| 18 | Whether a rate conflict blocks saving an entry | No: save, flag, alert |
| 19 | Behavior of a rule saved with an earlier effective date than existing entries | Uninvoiced entries are re-valued by date; closed invoices are not; conflicts are flagged |
| 20 | Windows and Linux timing | Later |
| 21 | Open-source license and business model (paid tiers) | Open; does not block the first milestone |
| 22 | Who may create API keys | The account holder, once API access exists |
| 23 | Client contact fields | Name only |
| 24 | Hub discovery and moving a hub to a new name | Phase 3 |
| 25 | Ownership transfer: what moves with the project (rates, clients, categories, shared tags, invoices, currency) | Open. Closed invoices stay as history |
| 26 | Rate for a former owner who stays as a project manager or collaborator | The new owner must set a base rate before accepting; otherwise the former owner's entries show "no rate" |
| 27 | Queued edits from before a transfer | Evaluated under the permissions in force when the hub receives them |
| 28 | Whether owner-only operations can ever be explicitly delegated | No, in the first milestone |
| 29 | Permission override precedence, and whether project managers may grant the manager role | Overrides beat presets; managers cannot grant the manager role |
| 30 | Whether non-overlapping edits to one record merge automatically | Yes (§13.3) |
| 31 | Who resolves a conflict on an owner-authority record when a delegate is involved | The owner |
| 32 | Cross-hub replication of amendments and conflicts | Phase 4 |
| 33 | Whether the hub warns the owner when a recorded offset disagrees with the hub's own timezone data | Optional warning only, never a rejection |
| 34 | Which timezone times are shown in by default | The author's recorded zone, labelled when it differs from the viewer's |
| 35 | Display of the end of a piece at midnight (24:00 or 00:00) | 24:00 |
| 36 | Whether the client uses bundled or operating-system timezone data | Open; the version used is recorded either way |
| 37 | Default zone for entries made after the fact, and how the zone is shown (§8.5) | The device's current zone, with a visible label that changes it |
| 38 | Automatic split of a running timer when the system zone changes, and the piece-boundary reason (§8.5) | Silent split, release 4 |
| 39 | A project merge for two installations whose projects overlap | Not in the first milestone |
| 40 | Where and how a hub is created (§13.5) | A server with a domain, one-command setup; other options in docs/architecture.md §5.9 |
| 41 | Moving an installation from one hub to another | Disconnect and join again; open |
| 42 | Sign-in method for a hub without a domain (device pairing) | Open; passkeys only for now |

---

# 23. Changes from Concept v1 and Heritage

## 23.1 Where v1 concepts went

| Concept v1 | Concept v2 |
|---|---|
| Central server; online only | Local-first app; hub optional |
| Admin creates users and sees all | Hub operator creates accounts; project owners see their projects |
| Admin owns clients and projects | Project owner |
| Admin assigns projects and categories per user | Project membership and capabilities; categories enabled per project |
| Admin sets rates; client charges not tracked | Project owner sets each member's rates |
| Admin edits any entry | No one rewrites another's entry; owner amends |
| Server-owned timer | Device timer, reconciled at sync |
| Super-admin command line for admin accounts | Hub operator command line |
| Admin password and TOTP | Open (§22 #4) |
| API keys per user | Same, via a hub |
| Reserved `lockedAt` | Contributor and client invoices (§12) |
| Server-side audit log | Append-only history on every record, plus the hub's log |
| Core and personal tags | Shared (owner-managed) and personal tags |
| Optimistic concurrency on edits | Version-based conflict detection with explicit, manual resolution (§13.3) |
| Server-side rules | A shared Rust domain core used by the app and the hub |

## 23.2 What carried over unchanged

The three ways to record time, whole-minute precision and `hh:mm`, timezone capture, midnight splitting, validation rules, overlap warnings, the rate-resolution and conflict model, immutable effective-dated rates, row-level editing, translation, tags, archive-over-delete, and the design philosophy.

## 23.3 Heritage

A prototype was built in April–May 2024 as a single-user application, with a database, an HTTP API and a browser interface. It was never deployed and holds no data worth migrating. Its repository will be reviewed during the technology discussion. Its reconstruction is kept in [docs/brief-v1.md](docs/brief-v1.md).
