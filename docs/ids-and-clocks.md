# IDs and clocks (decision D4)

**Status:** Decided 3 November 2026 for P0. Implemented in `crates/core/src/id.rs`.

## Identifiers

Format: `<uuid v7>@<origin>`, for example `0198f3c2-5b1e-7a40-9c11-2f6e0d7b8a55@i3f9a1c2b4d70`.

- **Created on the client.** No round trip is needed, so recording works offline.
- **Never reused.** A UUID v7 holds a 48-bit millisecond timestamp and 74 random or counter bits.
- **Roughly time-ordered.** The UUID comes first, so plain string order is creation order for one origin.
- **Origin-aware.** The suffix names the installation (later, the hub) that minted it. An installation's origin is a random token created at first run.
- **Validated.** Parsing accepts only v7 UUIDs and origins of 1 to 32 characters from `a-z`, `0-9`, `-`.

Different installations cannot collide because their origins differ; two generators with the same origin are still separated by the UUID's random bits (tested).

## Ordering of one person's edits

Every record has an integer **revision** that increases by one with each change. The store refuses a write unless its revision is exactly the stored one plus one. The device clock (`updated_at`) is informational and is never used to order edits. The hub-assigned order that sync will need is a release 2 question (architecture §6.3).

## Not decided here

Hub-issued identifiers and the claiming of a local identity by a hub account (concept §3.3) arrive with release 2.
