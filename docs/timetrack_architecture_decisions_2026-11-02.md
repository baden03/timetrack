# TimeTrack — Architecture Brainstorming Decisions

**Discussion date:** 2 November 2026  
**Reference:** https://github.com/baden03/timetrack  
**Status:** Reconstruction from recoverable conversation context, **not** a verbatim transcript or approved final architecture.

## 1. Product direction

TimeTrack is intended to grow beyond a single centralized, multi-user web application into a **local-first, open-source time-tracking system** with optional synchronization and collaboration through independently operated hubs. A standalone installation must remain useful without signing into any hub or maintaining a network connection.

**Agreed principles**

- Every installation is a peer in terms of its ability to create and own projects. There is no privileged primary client installation.
- Any user can create a project and invite other users into it, subject to relevant project permissions and hub federation policies.
- A user may have more than one device.
- The hub is optional for standalone time tracking; connecting to a hub enables device synchronization and shared-project collaboration.
- Hub-hosted backup and collaboration were discussed as potential paid tiers, without making standalone use dependent on those tiers.
- Hubs should be open-source and independently operable. **There is no global master hub.**

## 2. Deployment and evolution: four phases

### Phase 1 — Standalone local application

- An independently usable local time-tracking installation.
- No hub or internet connection required for core tracking.
- Users can create and own their own projects.
- Local data ownership and future synchronization should be considered in the data model from the outset.

### Phase 2 — Single-hub synchronization and collaboration

- Users may sign into a hub to synchronize multiple devices.
- Projects may be shared with invited collaborators.
- Hub queues/retains updates intended for disconnected participants and delivers them when they reconnect.
- Participants eventually converge on the shared information they are entitled to receive, even when temporarily offline.
- Exact concurrent-editing and conflict-resolution policy was **not finalized**.

### Phase 3 — Independent hubs

- Multiple independent operators can run their own hubs.
- A hub is not subordinate to a centrally operated service.
- Users and their devices can be associated with different hubs.

### Phase 4 — Hub federation

- Trusted hubs can communicate so people hosted on different hubs can collaborate.
- Federation is an explicit later phase, not a prerequisite for the standalone product.
- No master registry or controlling hub was agreed.

## 3. Federation and trust

- **Hub-to-hub trust must be explicit and directional**: Hub A trusting Hub B does not automatically mean Hub B trusts Hub A.
- Hub owners/administrators establish and control trust and policy.
- Once a hub administrator has authorized federation within a policy, individual users should be able to collaborate across those trusted hubs automatically, without requiring a separate administrator approval for every interaction.
- Trust between hubs is an enabling condition, **not** a replacement for the project-specific invitation and permission model.

**Still to design:** hub identity, trust establishment/revocation, cross-hub user identity, discovery, transport security, event routing, permissions enforcement, and what happens to queued changes when trust is revoked.

## 4. Project ownership and collaborative time data

- **Initial model: individual project ownership.** The project owner has ultimate project-level authority.
- Collaborators contribute their **own time entries** to projects shared with them.
- A collaborator's original recorded entry stays under that collaborator's control. **Even the project owner cannot directly rewrite it.**
- If the owner needs a different interpretation for their own records or reports, they can create a **separate amendment, note, or superseding record** that preserves the original.
- Owner-side amendments take effect **immediately in the owner's reporting**.
- The amendment must **not silently alter or propagate a rewritten entry back to the collaborator's own records**.
- The resulting design must distinguish the collaborator's original assertion from the owner's accounting/reporting adjustment and preserve provenance.

**Still to design:** amendment schema and linkage, audit semantics, rate of change to amendments, visibility of amendments to other users, and cross-device/cross-hub merge behavior.

## 5. Roles, permissions, and invitations

The discussion initially described three **project-scoped role concepts**:

| Role concept | Default information visibility / responsibilities |
|---|---|
| Collaborator | See their own time entries, not other collaborators' time. |
| Project manager | See all collaborators' time on that project; may invite users and assign collaborator or project-manager roles. |
| Owner | See everything in the project and retain project ownership. |

**Important refinement agreed later:** Avoid hard-coding all authorization into these three inflexible roles. Model access as **configurable project-level capabilities**, using roles as convenient presets and allowing **per-person overrides**. This should support delegation without forfeiting individual ownership.

- Project managers can be empowered to invite and manage collaborators and other project managers.
- Permission checks must be enforced consistently, including for synchronized changes and future API clients.
- The collaborator's own-time visibility restriction should apply across devices and hubs.

**Still to design:** full capability matrix, whether owners can delegate every owner power, invitation lifecycle, role/capability changes while devices are offline, and how revocations affect already-cached data.

## 6. Rates and accounting authority

- **Only the individual project owner defines rates**, under the initial ownership model.
- Collaborator-provided time and owner-defined accounting/rates are separate domains of authority.
- Owner-side amendments can affect the owner's reports while leaving the collaborator's underlying entry untouched.

The existing product specification also describes effective-dated rates, inheritance, conflict handling, and audit history. Their exact incorporation into the new local-first/federated architecture remains a detailed design task.

## 7. Synchronization principles and unresolved complexity

**Discussed/accepted:**

- Offline operation is supported for the standalone app and temporarily disconnected hub participants.
- A connected hub provides asynchronous forwarding/queuing so collaborators who were offline can eventually receive authorized updates.
- Multi-device use is a first-class scenario.

**Open design questions, not settled decisions:**

- Which records each participant or hub is authoritative for, and how ownership is proven.
- Global identifiers for users, clients, projects, time entries and amendments across hubs.
- How to reconcile simultaneous edits and deletions; whether to use append-only events, version vectors, or other approaches.
- Handling deleted/revoked shared projects, invitations, devices, and federation relationships while peers are disconnected.
- What a hub may retain, inspect, or modify in transit.
- Recovery after lost devices and reconstitution of local state.

## 8. Relationship to the repository specification

The GitHub repository was discussed as containing a **Product Specification v2.0**, rather than an implemented application. Its then-described scope was a centralized, desktop-browser-first system with:

- Users, clients, projects, categories, tags and time entries.
- Passkey authentication and admin-controlled invitations.
- User-specific access controls.
- Effective-dated hourly rates with conflict handling and audit history.
- A future public API sharing the application's business rules.
- No offline sync, invoicing, or complete reporting module in its initial v1 scope.

**These requirements are not automatically the final architecture of the expanded concept.** The earlier specification's central-server/no-offline assumptions need deliberate reconciliation with the newly discussed local-first architecture. Likewise, admin-controlled onboarding for a central deployment should not be presumed to supersede the later project-based invitation model.

## 9. Technology discussions — suggestions, not decisions

Ideas raised earlier included:

- Modular monolith rather than microservices.
- Node.js and TypeScript backend; React and TypeScript frontend.
- PostgreSQL relational database.
- WebAuthn/passkeys, with identity and permissions separated from the authentication provider.
- Docker Compose on a VPS.
- Shared business-service logic for UI and future APIs.
- Server-authoritative time and rate calculations and atomic mutation/audit transactions in a centralized design.

**None of these technologies was selected.** In particular, server-authoritative assumptions need revision where offline-first installations and independent hubs have equal standing.

## 10. Suggested next architecture workshop topics

1. **Data authority and identity:** specify unique identities for people, devices, hubs, projects, time records and accounting amendments.
2. **Local storage and event model:** design standalone persistence so synchronization can be added without a rewrite.
3. **Shared-project protocol:** invitations, subscriptions, visibility filters, reliable delivery and revocation.
4. **Role/capability model:** define defaults, project-level permissions and per-person overrides.
5. **Original vs. amendment semantics:** define how owner reports derive their effective figures from immutable contributor entries plus owner adjustments.
6. **Hub-to-hub federation:** directional trust, identity and discovery, permitted data movement and revocation.
7. **MVP boundary:** distinguish what must ship in Phase 1 from infrastructure deliberately designed for Phases 2–4.

## 11. Decision ledger

| Topic | Status | Outcome |
|---|---|---|
| Standalone mode | Agreed | Local-first, offline-capable, open-source client. |
| Hub requirement | Agreed | Optional; enables sync and collaboration. |
| Installations | Agreed | Equal ability to create/own projects. |
| Multiple devices | Agreed | Supported through hub sync. |
| Hub hierarchy | Agreed | Independent hubs; no master hub. |
| Federation sequencing | Agreed | Fourth phase after standalone, single hub and independent hubs. |
| Hub trust | Agreed | Explicit, directional, administrator-controlled. |
| Cross-hub user actions | Agreed in principle | Automatic within established hub federation policy. |
| Project ownership | Agreed for initial version | Individual project owner. |
| Rates | Agreed | Owner defines them. |
| Collaborator records | Agreed | Owner may view, not rewrite originals. |
| Owner corrections | Agreed | Separate amendments; affect owner reporting only. |
| Permissions | Agreed direction | Project-level capabilities with role presets/per-user overrides. |
| Sync transport and conflict algorithm | Open | Needs design. |
| Frameworks and database | Open | Only candidates discussed. |

---

*This summary reconstructs the discussions and distinguishes confirmed preferences from proposals and unanswered questions. It is intended as an architecture working document, not as a verbatim historical record.*
