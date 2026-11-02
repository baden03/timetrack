# TimeTrack — Product Specification

**Version:** 2.0 (draft for review)
**Date:** November 2026
**Status:** Specification only. Technology stack and implementation plan are not yet decided.
**Supersedes:** the 2024 prototype reconstruction, preserved at [docs/brief-v1.md](docs/brief-v1.md).

This document is technology-agnostic. It states what the product does and the rules it must follow, not how it is built.

---

# 1. Purpose and Scope

TimeTrack is a multi-user time log for a small team of freelancers and contractors working on shared client projects. It replaces the per-client, per-project Google Docs currently used to record hours.

It starts as a tool for one person (the admin) and grows to include multiple collaborators. Each person logs their own time, and the admin sees everything and can value that time at each person's hourly rate.

**Design philosophy.** Recording and correcting time must take as little effort as possible. A user should be able to open the app, pick a project, enter or start a time record, and see the day's total immediately. Administration stays out of the way. Priorities: **speed, simplicity, accuracy, minimal interruption.**

## 1.1 In scope for version 1

- Admin-created user accounts with roles, project and category assignment
- Clients, projects, categories, and tags (admin-maintained core tags plus personal tags any user can create)
- Three ways to record time: start/end, duration only, live timer
- Automatic splitting of entries that cross midnight
- Per-user hourly rates with override rules and full rate history
- Billable / non-billable flag
- Search across all entries, with filtering
- Complete, append-only audit log
- Per-user preferences (timezone, date format, clock format) and system settings (currency)
- Light and dark appearance
- A translatable interface (English by default; see §12.4)
- A service layer shaped so that a public API can be added without rework (see §17)

## 1.2 Explicitly out of scope for version 1

- Reporting module (added later; the data model must support it)
- Invoicing, and tracking what clients are charged
- Cost-versus-billing rate distinction (only the user rate exists in v1; see §8)
- Project or task budgets
- Expense tracking
- Data export and import
- Offline use and mobile-specific apps; desktop browsers are the primary platform
- Email notifications of any kind
- A public API (designed for in version 1, built later; see §17)
- Interface languages other than English (the mechanism is in scope, additional translations are not)

## 1.3 Primary platform

Desktop browsers. Tablet and phone layouts must remain usable but are not optimized for editing.

---

# 2. Decisions Log

| Topic | Decision |
|---|---|
| Users | Multi-user. Accounts created by an admin only. Unique email per user. |
| Visibility | Users see only their own entries. Admin sees all. |
| Project access | Admin assigns users to projects. Users see and log only assigned projects. |
| Category access | Admin assigns categories to users. Archiving a category is global. |
| Tags | Core tags maintained by the admin and available to everyone. Any user may also tag an entry with any tag they type (a personal tag, suggested only to its creator and visible to the admin). Names up to 40 characters, up to 10 tags per entry. |
| Language | All interface text is translatable. Default English. Per-user language preference. |
| Code language | All code, comments and developer documentation are in English. |
| API | A future add-on, but designed for from the start. API keys belong to a user and act with that user's permissions. |
| Time recording | (a) start and end, (b) duration only, (c) live timer. |
| Display of durations | `hh:mm` everywhere. |
| Precision | Whole minutes. No rounding, no 15-minute rule. |
| Midnight | Entries crossing midnight are split automatically into linked entries. |
| Timezone | One IANA timezone per user, captured on every entry. |
| Description | Required. |
| Project date | Removed. Derived from the first and last entries. |
| Rates | Per-user hourly rate, required at user creation, immutable, effective-dated, with optional overrides by client, project, category. |
| Currency | Global system setting. Default EUR. |
| Rate conflicts | Never resolved silently. The admin is alerted and decides. |
| Delete | Clients and projects: archive, or hard delete in a confirmed danger zone. Categories and tags in use can only be archived. |
| Editing | Row-level in-place editing. |
| Concurrency | Optimistic concurrency on all edits. |
| Audit | Detailed append-only audit log in version 1. |
| Notifications | In-app only. Only for a timer left running from a previous day. |
| Sign-in | Passkeys for everyone. Google sign-in optional (decided later). Admin may also use email and password. |
| Break-glass | A command-line super-admin procedure that can update admin email, and issue new admin invite link. No web login. |
| Recovery | Admin re-issues an invite link. |
| Onboarding | Admin copies an invite link and sends it personally. No outgoing email. |
| Timer | One running timer per user. |
| Entry window | Default view shows the last 60 days (system setting). Search covers all entries viewable by the user |

---

# 3. Roles and Access

## 3.1 Roles

| Role | Capabilities |
|---|---|
| **User** | Log, edit and delete own entries on assigned projects and categories. Run own timer. View own entries, own rates and own values. Edit own preferences. |
| **Admin** | Everything a user can do, plus: manage users, clients, projects, categories, tags, assignments, rates and system settings. View and edit all users' entries. View the audit log. Resolve rate conflicts. |
| **Super-admin** | Not an application account. A command-line procedure on the server (see §4.5). Creates admin accounts, updates admin email, and re-issues admin invites. |

Only the super-admin procedure creates admins. Admins create ordinary users.

## 3.2 Visibility rules

- A user sees only their own entries and rates, never another user's.
- A user sees only projects and categories assigned to them.
- A user sees core tags and their own personal tags, never another user's personal tags. An admin sees any tag attached to an entry they can view.
- Existing entries remain visible to their owner even if the project or category is later unassigned or archived.
- An admin's project page lists all users' entries, grouped by date and then by user, with a user filter.

## 3.3 Enforcement

All access rules are enforced at the service layer. Hiding navigation items is never sufficient.

---

# 4. Authentication and Accounts

## 4.1 Account creation (invite flow)

1. The admin creates a user: email (unique, case-insensitive), display name, role, timezone, **base hourly rate**, project and category assignments.
2. The system produces a one-time invite link. The admin copies it and sends it to the person. The system sends no email.
3. The invitee opens the link and registers a sign-in method (§4.2).
4. The invite expires after a short fixed period and can be reissued by the admin.

## 4.2 Sign-in methods

| Method | Who | Notes |
|---|---|---|
| **Passkey** | Everyone | Baseline method. At least one method is always required. |
| **Google (OpenID Connect)** | Everyone, optional | Matched on the verified email address, then pinned to the provider's stable subject identifier. Whether v1 ships this is decided with the stack. |
| **Email + password** | Admin only | Requires a second factor (TOTP). Passwords are stored only as strong salted hashes. |

There is no self-registration and no public sign-up.

## 4.3 Sessions

- Long-lived sessions suited to a daily-use desktop tool (for example 30 days, sliding).
- Disabling a user or changing their role revokes all of their sessions immediately.
- Every sign-in, failed sign-in, sign-out, session revocation and credential change is recorded in the audit log.

## 4.4 Account states

- **Active**: can sign in.
- **Disabled**: cannot sign in; data retained.
- Users who own entries are never hard-deleted. Disable them instead. A user with no entries and no audit references may be removed by an admin with confirmation.

## 4.5 Break-glass: super-admin procedure

- A command run on the server, with no network-facing login.
- It can only:
  - create an admin account
  - update admin email address
  - issue a fresh admin invite link
  - nothing else
- It has no access to clients, projects, entries or rates.
- Every use is written to the audit log with the actor shown as the super-admin procedure.
- This is the only recovery path for an admin who has lost all sign-in methods.

---

# 5. Domain Model

Field lists are logical. Types and storage are left to the implementation.

## 5.1 User

`id`, `email` (unique), `displayName`, `role` (user | admin), `status` (active | disabled), `timezone`, `language`, `dateFormat`, `clockFormat` (24h | 12h), `apiAccess` (boolean, default off), `createdAt`, `lastLoginAt`.

## 5.2 Client

`id`, `name`, `status` (active | archived). Additional contact fields are deferred.

One client has many projects.

## 5.3 Project

`id`, `clientId`, `name`, `description`, `status` (active | archived), `billableDefault` (boolean).

Derived, not stored: first entry date, latest entry date, total time.

A project belongs to exactly one client. A client can be reassigned by an admin; the change is audited.

## 5.4 Category

`id`, `name`, `status` (active | archived).

A category describes a type of work. It carries no rate. Rates are defined separately (§8).

## 5.5 Tag

`id`, `name`, `kind` (core | personal), `ownerUserId` (personal tags only), `status` (active | archived).

- **Core tags** are maintained by the admin and available to every user.
- **Personal tags** are created by any user simply by typing a new tag on an entry. They are suggested only to their creator. Other users never see them. The admin can see all personal tags, in tag review and on entries.
- A tag name is 1–40 characters, trimmed, and unique ignoring case within the core set and within each user's personal set. If a user types a name that matches a core tag, the core tag is used.
- An entry may carry up to 10 tags.
- The admin can **promote** a personal tag to a core tag. Personal tags with the same name belonging to other users are merged into it, and their entries are re-linked. The change is audited.
- A user can rename or archive their own personal tags, and delete them when unused.

## 5.6 Assignments

- **UserProject**: which users may log time on which projects.
- **UserCategory**: which categories each user may select.

Removing an assignment never alters existing entries or rate rules.

## 5.7 Time Entry

| Field | Description |
|---|---|
| `id` | Unique identifier |
| `userId` | Owner |
| `projectId` | Project |
| `categoryId` | Category |
| `kind` | interval, duration, or running |
| `workDate` | Calendar date the work belongs to, in the entry's timezone |
| `timezone` | The owner's timezone at the time of entry |
| `startAt`, `endAt` | Absolute instants (interval and running kinds) |
| `minutes` | Duration in whole minutes. **Derived** from `startAt`/`endAt` for intervals; stored data for duration-only entries |
| `description` | Required text (required at stop for running timers) |
| `billable` | Boolean; defaults from the project, overridable |
| `tags` | Zero to ten tags, core or personal |
| `source` | Where the entry came from: web, or api (reserved for §17) |
| `externalRef` | Optional identifier supplied by an external tool, unique per user when present (reserved for §17) |
| `splitGroupId` | Shared by linked halves of a midnight split |
| `version` | Optimistic-concurrency counter |
| `createdAt`, `updatedAt` | Timestamps |
| `lockedAt` | Reserved for a future "billed / period locked" rule; v1 has no locking UI |

## 5.8 Rate Rule

See §8.

## 5.9 Audit Event

See §10.

## 5.10 System Settings

`currency` (default EUR; locked once any rate exists), `entryWindowDays` (default 60), `defaultTimezone`, `defaultLanguage` (default English), default `dateFormat` and `clockFormat` for new users.

---

# 6. Time Recording

## 6.1 The three ways to record

**A. Start and end.** The user enters a start time and an end time (and optionally a date, default today). Duration is calculated.

**B. Duration only.** The user enters a duration as `hh:mm` and a date (default today). Start and end are left empty. The entry shows "–" in the From and To columns.

**C. Live timer.** See §7.

All three produce the same kind of entry, appear in the same tables, and can be edited later.

## 6.2 Required and optional fields

Required: project, category, description, and either (start and end) or a duration. Optional: date (default today), tags (a core tag, or any new tag typed in), billable flag (defaults from the project).

## 6.3 Dates and timezones

- Entries default to today in the user's timezone.
- Entries may be back-dated and existing entries may be modified.
- Entries dated or ending in the future are rejected.
- `workDate` is a plain calendar date computed in the entry's stored timezone. Changing a user's timezone setting affects only entries created afterwards; existing entries keep their original timezone and date.
- Where a wall-clock time is ambiguous (the repeated hour at the end of daylight-saving time) the first occurrence is used.

## 6.4 Validation

- End must be after start. Zero and negative durations are rejected.
- An interval may not exceed 24 hours.
- A duration-only entry must be at least one minute and at most 24 hours.
- Category and project must be assigned to the user and not archived (archived items cannot be chosen for new entries).
- **Overlaps:** if an interval overlaps another interval of the same user, the system warns and allows the save. Overlaps are never checked across users, and duration-only entries never take part.
- All rules are enforced on the server, regardless of client-side checks.

## 6.5 Midnight splitting

When an interval crosses midnight in the entry's timezone, the system splits it automatically into one entry per calendar day:

- Example: 23:00 to 02:00 becomes 23:00–24:00 on the first day and 00:00–02:00 on the next day.
- The pieces share a `splitGroupId` and show a visible link indicator.
- Description, category, project, tags and billable flag are the same on each piece.
- The pieces are treated as one logical span when edited: the user changes the start of the first piece or the end of the last piece, and the system re-splits. If the edit removes the midnight crossing, the pieces merge into one entry.
- Deleting one piece deletes the whole group, after a confirmation that says so.

## 6.6 Keyboard behaviour (new entry form)

The form is a compact single row on wide screens and wraps on narrow ones.

- **Enter** in the description field submits. **Shift+Enter** inserts a new line.
- After saving, the form clears, the table and daily totals update without reloading, and focus returns to the first field.

**Smart defaults** (to minimize typing): the start time is proposed as the end of the user's previous entry that day, and the category as the last one used on this project.

## 6.7 Feedback

The user receives clear feedback on success, validation failure, server rejection, network failure and version conflicts. Failed saves never discard what the user typed.

---

# 7. Live Timer

- Each user may have at most **one** running timer.
- Start: choose project and category (description and tags optional at this point). The system records the start time. The timer is stored on the server, so it survives closing the browser and works across devices.
- While running, a persistent indicator with elapsed time is visible in the header.
- Stop: the user confirms the description (required) and may adjust tags and billable flag. The system records the end time, and the entry becomes an ordinary interval. If it crosses midnight, it is split (§6.5).
- Start and stop times are captured to the minute, so durations are always whole minutes.
- A start control may appear on each dashboard project card.

## 7.1 Timer left running from a previous day

At the next sign-in or page load, the user sees an in-app prompt. They enter the real stop time and the description, and the entry is saved (and split if needed). They can also discard the timer. This prompt is the only notification in v1.

---

# 8. Rates and Valuation

## 8.1 Concept

A **rate** is the hourly value of one user's time. What the client is ultimately charged is not tracked. In v1 there is a single rate family, the user rate. Cost and billing rates are possible later extensions.

Only the admin defines rates. A user sees only their own rates and values; nobody sees another user's.

## 8.2 Rate rules

A rate rule contains:

- `userId` (always required)
- optional `clientId`, `projectId`, `categoryId`
- `amount` (hourly, in the system currency)
- `effectiveFrom` (a date; the first rule for a user applies "from the beginning")
- who created it and when

Rules are **immutable**. A rate is never edited. To change one, the admin adds a new rule with a later `effectiveFrom`. The previous rule is considered ended by the new one.

Every user has a **base rate**: a rule with only `userId`. It is required when the user is created. There is no system-wide default rate.

## 8.3 Resolving the rate for an entry

For an entry with a user, client (via its project), project, category and `workDate`:

1. For each distinct scope (the combination of user, client, project and category fields that are set), take the rule with the latest `effectiveFrom` on or before the `workDate`.
2. Keep the scopes whose set fields all match the entry.
3. If one scope is **more specific** than all others, use its rate. Scope A is more specific than scope B when everything A matches is also matched by B. A project rule is therefore more specific than its client's rule, and a rule with more fields set is more specific than one with a subset of those fields.
4. If no scope is more specific than all others, there is a **conflict** (§8.5).
5. The base rate is the fallback when no override matches.

Rates are looked up by the entry's work date, not by when it was entered, so back-dated entries use the rate in force on that date.

## 8.4 Entry value

`value = minutes × hourlyRate / 60`, rounded half-up to the smallest currency unit **once per entry**. Totals are always the sum of entry values, so a total always equals the sum of the visible lines.

Billable status does not change value. It only marks hours as billable to the client for later reporting.

## 8.5 Conflicts

A conflict exists when two or more matching rules overlap and neither is more specific than the other. Example: for Doug, "Development" at €120 and "client Acme" at €110 both match Doug logging Development for Acme.

- Conflicts are an edge case. They are never resolved silently or by a hidden priority order.
- When the admin saves a rule that would create a conflict, the system says so and asks for the rate of the combined case (Doug + Acme + Development), which is more specific than both and resolves it.
- If an entry is saved while a conflict exists (for example, due to a back-dated rule), the entry is flagged **rate conflict**, excluded from value totals, and the admin sees an alert in the admin navigation until it is resolved.
- Every conflict resolution is audited.

## 8.6 Rate table

The admin has a table per user showing the **current** rate and all overrides, with each rule's effective date and an expandable history.

## 8.7 Worked examples (test vectors)

Base rates: Doug €100/h, Steffen €70/h.

| Case | Result |
|---|---|
| Doug, 90 min, base rate only | 90 × 100 / 60 = **€150.00** |
| Steffen, 20 min | 20 × 70 / 60 = 23.333… → **€23.33** |
| Steffen, 25 min | 25 × 70 / 60 = 29.1666… → **€29.17** |
| Doug base €100 until 2027-01-01, then €110 (new rule dated 2027-01-01). Entry on 2026-12-31 | **€100** rate |
| Same, entry on 2027-01-01 | **€110** rate |
| Doug + Development = €120, Doug + project Acme/Website = €115. Doug logs Development on Acme/Website | **Conflict** (neither scope contains the other) |
| Add Doug + Acme/Website + Development = €130, then log the same entry | **€130** rate |
| Doug + client Acme = €110, Doug + project Acme/Website = €115. Doug logs on Acme/Website | **€115** (project is more specific than client) |

---

# 9. Screens

## 9.1 Dashboard

- A responsive grid of cards for the user's active, assigned projects.
- Card: project name, client name, description (truncated gracefully). The whole card is a link; a timer start control may be present.
- Keyboard and pointer accessible.
- Empty state: for a user with no assigned projects, a message to ask the admin; for an admin, a path to create the first client and project.

## 9.2 Project time page

Title shows the project name. Two parts:

1. **New entry form** (§6), with a mode switch for start/end, duration, or timer.
2. **Entries**, grouped by date, newest first, one table per date. The group header shows the date in the user's date format and the day's total in `hh:mm`.

Only the last `entryWindowDays` days are loaded by default, with a control to load older entries. Admins see all users' entries, with a user column and filter.

Columns: From, To, Duration (`hh:mm`, read-only), Category, Description, Tags, Billable, and (for the entry's owner and admin) Value. From and To show "–" for duration-only entries. A split entry shows its link.

Empty state: "No time entries have been recorded for this project." An empty project is a normal state, not an error.

## 9.3 Row-level editing

- Activating a cell (click or double-click) puts the **whole row** into edit mode, with focus in the chosen cell.
- **Enter** saves, **Esc** cancels. Save and Cancel buttons are always visible.
- The row is validated as a whole and saved in a single update (one audit event, one version change).
- **Tab** moves between fields within the row; leaving the row without saving asks whether to keep or discard changes.
- Duration is read-only on interval rows. Editing start and end recalculates it, splits or merges across midnight as needed, and updates the daily totals.
- Touch devices get an explicit Edit button on each row that opens the same mode.
- A save error keeps the row open with the edited values, shows the error, and lets the user retry or cancel. The last saved values are never overwritten.
- If another save has changed the entry since it was loaded (version conflict), the user is told and offered a reload.
- Editable fields: date, start, end (or duration for duration-only entries), category, description, tags, billable flag.

## 9.4 Search

- Searches **all** entries, not only the recent window.
- Free-text search over descriptions, with filters for client, project, category, tag, billable flag, date range and (admin only) user.
- Results show the total time and, where the viewer may see it, the total value for the filtered set.
- Results open for editing in the same way as the project page.

## 9.5 Administration screens (admin)

- **Users**: create, edit, disable, re-issue invite, assign projects and categories, rate table (§8.6).
- **Clients**, **Projects**, **Categories**: create, edit, archive, hard delete where permitted.
- **Tags**: create, edit and archive core tags; review personal tags with their owners and usage; promote a personal tag to core.
- **System settings**: currency, entry window, defaults for new users.
- **Rate conflicts**: list of unresolved conflicts with a resolve action.
- **Audit log**: browse and filter (§10).

## 9.6 Navigation

Persistent navigation: Dashboard, Projects, Search, and (admin) Administration. A user menu with profile and preferences, theme switch, and sign-out. The current location is identifiable, and menus are keyboard accessible.

---

# 10. Audit Log

## 10.1 Properties

- **Append-only.** No screen or operation edits or deletes audit events.
- Detailed enough to reconstruct who changed what, when, and from what to what.

## 10.2 Event content

Each event records: timestamp, actor (user, or the super-admin procedure), the session or the API key used, the entity type and ID, the action, the previous and new values (field-level changes), and a reason or comment where one was supplied.

Events store language-neutral codes and values, not rendered sentences, and are displayed in the viewer's language (§12.4).

## 10.3 What is logged

- All create, update, archive, restore and delete operations on entries, clients, projects, categories, tags, assignments, rate rules, users and system settings
- Rate-conflict detection and resolution
- Sign-ins, failed sign-ins, sign-outs, credential changes, invites issued and consumed, session revocations
- Super-admin procedure use
- Timer start, stop, discard
- API key creation and revocation, and every write made with a key

## 10.4 Hard deletes

A hard delete writes a full snapshot of everything removed (for example, the client, its projects and every entry) into the audit log before the data is removed, so the history of what was destroyed survives.

## 10.5 Access

Admins can view and filter the whole log. A user can view the history of their own entries.

Retention and any personal-data handling for the log are open (see §15).

---

# 11. Archive and Delete

| Item | Archive | Hard delete |
|---|---|---|
| Client | Yes (default). Hides it and its projects from active lists; history remains. | Admin only, danger zone: lists the number of projects and entries that will be destroyed and requires explicit confirmation. Removes projects and entries atomically. |
| Project | Yes (default). | Same, for its entries. |
| Category | Yes (never hidden from existing entries). | **Never** if any entry or rate rule references it. |
| Tag | Yes. Archived tags are no longer suggested; existing entries keep them. | Never while in use. A personal tag may be deleted by its owner when unused; a core tag by an admin when unused. |
| Entry | n/a | Owner or admin, with confirmation, unless locked (reserved). Audited with a snapshot. |
| User | Disable | Only if the user has no entries and no audit references. |

Archived items can be restored by an admin. Deletes that touch several records are atomic.

---

# 12. Preferences and Appearance

## 12.1 User preferences

- Language (default English; see §12.4)
- Timezone (IANA)
- Date format (default `DD.MM.YYYY`; others available)
- Clock format (default 24-hour; 12-hour available)

Duration is always displayed `hh:mm`. All dates and times are shown in the viewing user's formats and timezone. Language and date format are independent settings: English text with German-style dates is the default combination.

## 12.2 Theme

Light and dark themes, a header switch, persisted per device, defaulting to the system preference. Visual styles use centralized theme tokens; no hard-coded colors.

## 12.3 Responsive and accessible

- Desktop first; usable on tablet and phone.
- Keyboard navigation, visible focus, labelled inputs, accessible error messages, sufficient contrast, semantic table markup, accessible dialogs.
- Every pointer-only interaction has a keyboard alternative.

## 12.4 Language and translation

- All code, comments, identifiers and developer documentation are written in English.
- Every piece of user-visible text lives in translation catalogs, separate from code. No screen contains hard-coded text.
- Adding a language means adding a catalog. It requires no code change.
- English is the default. The admin sets the default language for new users, and each user can choose their own.
- A missing translation falls back to English. A raw translation key is never shown to a user.
- Messages that contain values or counts use the translation system's parameters and plural rules. Sentences are never assembled by joining fragments, because word order and plural forms differ between languages.
- The server reports errors and validation failures as stable codes plus parameters. The interface turns them into translated text. The future API (§17) uses the same codes.
- The audit log stores codes and values, so each viewer reads it in their own language.
- User-entered data (client, project, category and tag names, descriptions) is shown as entered and is not translated.
- Layout tolerates longer text and uses direction-neutral styling, so that right-to-left languages can be added later without redesign.

---

# 13. Calculations and Test Vectors

## 13.1 Rules

- Duration of an interval = whole minutes between `startAt` and `endAt` (real elapsed time).
- Display: `hh:mm`, with hours not capped at 24 (for example `126:30`).
- Daily total = sum of the minutes of that date's entries for the viewed user (or per user in the admin view).
- Totals are computed from minutes, never from displayed values.
- Value follows §8.4.

## 13.2 Examples

| Case | Result |
|---|---|
| 09:15 to 12:45 | 210 min = **3:30** |
| Day with 09:00–11:30, 11:45–13:00, 14:00–17:00 | 150 + 75 + 180 = 405 min = **6:45** |
| Edit 09:00–11:00 to 09:00–11:15 | Duration 2:00 → **2:15**; day total increases by 0:15 |
| 23:00 to 02:00, Europe/Berlin, starting 2026-11-02 | Two entries: 2026-11-02 23:00–24:00 (1:00) and 2026-11-03 00:00–02:00 (2:00); both share a group; combined **3:00** |
| Edit the split span's end from 02:00 to 23:30 | Pieces merge into one entry on 2026-11-02, **0:30**; the group no longer exists |
| 01:30 to 03:30 local time on 2026-03-29, Europe/Berlin (clocks jump 02:00 to 03:00) | Real elapsed **1:00** |
| Duration entry 1:30 on 2026-11-02 | Entry with no start or end, **1:30** |
| End before start (10:00 to 09:00) | Rejected |
| Start equals end | Rejected |
| Duration longer than 24 hours | Rejected |
| End time in the future | Rejected |
| Overlap with the user's own existing entry | Warning; save allowed |
| Overlap with another user's entry | No warning |

---

# 14. Non-Functional Requirements

## 14.1 Security

- Authenticated access to all private data; authorization on the server for every operation.
- Secure session handling, encrypted transport, protection against injection, cross-site scripting and request forgery.
- No plain-text secrets. Passwords (admin only) only as salted, strong hashes with a second factor required.
- Rate limiting and lockout on repeated failed sign-ins.
- Secrets and credentials kept out of source control.

## 14.2 Reliability and integrity

- Referential integrity between all related records.
- Multi-record operations are atomic.
- Failed saves never silently lose user input.
- Optimistic concurrency on every update.
- Server-side validation of every rule; the client is advisory only.

## 14.3 Performance

- Interactive operations feel immediate.
- The default view is bounded by the entry window; search and long histories must remain responsive.

## 14.4 Maintainability

- Business rules (time, splitting, rates) live in one place and are shared by every code path.
- Validation is centralized and reused by create and edit.
- Table columns and form fields are declared in configuration, not in the order data happens to arrive.
- The user interface and any future API are both clients of one application service layer. No business rule exists only in screen code (see §17.2).
- All code and comments are in English (see §12.4).

## 14.5 Operations

- Backups with a tested restore procedure.
- Monitoring and error reporting.
- HTTPS everywhere.
- A documented procedure for the super-admin command.

## 14.6 Testing

Automated tests are required for:

- Duration, daily total, `hh:mm` formatting and value calculation, using the vectors in §8.7 and §13.2
- Midnight splitting, merging and editing of split groups, including daylight-saving days
- Rate resolution, effective dates, specificity and conflict detection
- Timezone handling, including a user changing timezone
- All validation rules, on the server
- Access control: user versus admin visibility, unassigned projects and categories, disabled users
- Archive and delete rules, including blocked deletes and atomic cascades
- Audit log completeness for each operation type
- Authentication flows: invite, passkey, admin password with second factor, session revocation, break-glass invite
- Timer: single timer, stop, stale-timer prompt, stop across midnight
- Optimistic concurrency conflicts
- Tags: core versus personal visibility, name matching, promotion and merge
- Translation: every catalog key used by the interface exists in English, missing translations fall back, plurals and parameters render correctly
- Accessibility of keyboard flows, and both themes

---

# 15. Open Items

These remain undecided. Each has a working default that the specification above already assumes.

| # | Item | Working default |
|---|---|---|
| 1 | Google sign-in in v1 | Decide with the stack. Passkeys are the baseline either way. |
| 2 | Second factor for admin password | TOTP required. |
| 3 | Super-admin command | Depends on server shell access in the chosen hosting. |
| 4 | Whether users see the value of their own entries | Yes. |
| 5 | Client contact fields | Name only in v1. |
| 6 | First additional interface language | None in version 1; English only, with the mechanism ready (§12.4). |
| 7 | Default timezone for new users | Europe/Berlin, set by the admin at creation. |
| 8 | Audit-log retention and personal data (users are individuals; entries and the log may hold names and IP addresses) | Keep indefinitely until a policy is set. |
| 9 | Statutory retention of billing-related records in Germany | Confirm with an accountant. Hard delete is admin-only, and a "locked" hook (`lockedAt`) is reserved. |
| 10 | Whether an unresolved rate conflict should block saving the entry | No: save, flag, alert the admin. |
| 11 | Maximum look-back for back-dating | Unlimited in v1. |
| 12 | Behaviour for a rule saved with an `effectiveFrom` earlier than existing entries | Entries are re-valued by date; any conflicts are flagged. |
| 13 | Who may create API keys | Admin enables API access per user (default off); the user then manages their own keys. |
| 14 | API key scopes | Two: read-only, and read plus write time entries. |
| 15 | Duplicate protection when an API client resubmits an entry | The same `externalRef` from the same user returns the existing entry instead of creating a second one. |

---

# 16. Future Work (not in version 1)

- Reporting module: totals by client, project, category, user and period; billable summaries; CSV export
- Data export and import; migration tools
- Cost rates and client billing rates alongside the user rate
- Period locking ("billed") on top of `lockedAt`
- Project and task budgets
- Invoicing
- Expense tracking
- Offline use and installable (PWA) or local-first apps
- Email notifications and reminders
- Additional sign-in providers
- The public API (§17)
- Additional interface languages

---

# 17. Planned API (designed for now, built later)

## 17.1 Intent

A developer who has built their own time-tracking tool should be able to connect it to TimeTrack with an API key: read the reference data they are allowed to use (clients, projects, categories, tags) and submit time against those clients and projects.

The API is not part of version 1 delivery. Version 1 must not make it hard to add.

## 17.2 Design constraints on version 1

- **One service layer.** Every operation the interface performs (list projects, create an entry, edit an entry, and so on) is an application operation that carries its own validation, authorization and audit. The interface and the future API are both clients of it. Midnight splitting, overlap warnings, rate handling and access control are therefore identical for both.
- **Stable identifiers.** Records have opaque identifiers that never change and are never reused.
- **Machine-readable errors.** Failures carry stable codes and parameters (§12.4).
- **Unambiguous data types.** Timestamps are absolute instants with an explicit offset. Durations are integer minutes. Money is an integer in the smallest currency unit plus a currency code.
- **Concurrency.** The entry `version` is part of the contract so a client can detect a conflicting change.
- **Lists** are paginated and filterable by date range, project and similar fields.
- **Provenance.** Entries record their `source` and may carry an `externalRef`, an identifier from the external tool (§5.7).
- **Audit.** Writes made through a key are attributed to the user and to the key (§10).

## 17.3 API keys

- A key belongs to one user and **acts as that user**. It sees exactly what that user sees, is bound by the same project and category assignments, and goes through the same validation and splitting. It can never do more than the user.
- A key has one of two scopes: read-only, or read plus write of time entries.
- The full key is shown once, at creation. Only a one-way hash is stored.
- A key has a label, an optional expiry date and a last-used timestamp. The owner or an admin can revoke it at any time. Disabling the user revokes their keys.
- API access is switched on per user by the admin (default off). The user then manages their own keys.
- Keys cannot manage users, credentials, assignments, rates or settings.
- Requests are rate limited. Key creation, revocation and every write are audited. Individual reads are not.
- Keys are separate from browser sessions and passkeys.

## 17.4 Intended capabilities (first release of the API)

| Area | Operations |
|---|---|
| Current user | Read own profile and preferences |
| Clients, projects, categories | List and read those the user may use |
| Tags | List core tags and the user's own personal tags |
| Time entries | List own entries (filter by date, project, category, tag); create, update and delete own entries using start and end, or duration |

Creating an entry with an `externalRef` that the user has already used returns the existing entry, so a client can safely retry. Rates and values are not exposed in the first release. The live timer is not exposed in the first release.

## 17.5 Contract and compatibility

- The API is versioned. A breaking change is a new version.
- The contract is published as a machine-readable description with examples.
- Deprecated versions are announced before they are removed.

---

# 18. Heritage

A prototype was built in April–May 2024 as a single-user application, with a database, an HTTP API and a browser interface. It was never deployed and holds no data worth migrating. Its repository will be reviewed after the technology discussion. The detailed reconstruction of that prototype is kept in [docs/brief-v1.md](docs/brief-v1.md).
