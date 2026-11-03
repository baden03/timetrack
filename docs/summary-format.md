# Monthly summary format (decision D5)

**Status:** Signed off by Baden on 3 November 2026. Implemented in `crates/core/src/summary.rs`.

The summary is plain text for one calendar month. It feeds an LLM outside TimeTrack that writes and translates the invoice text.

## Sample

```
TimeTrack summary — November 2026

ACME GMBH
=========

Website relaunch — 41:30
  Development — 28:45
    - Built the contact form and validation
    - Fixed Safari layout bug in the header
  Meetings — 9:15
    - Weekly sync with Anna
    - Kickoff call for the booking module
  Design — 3:30
    - Revised homepage hero layout after feedback

Client total: 47:30

Grand total: 59:45
```

## Rules

- **Grouping:** client, then project, then category. Names are sorted alphabetically, ignoring case. Items are grouped by identity, so two clients with the same name stay separate.
- **Totals:** `hh:mm` at every level, computed from minutes. The category totals add up to the project total, the project totals to the client total, and the client totals to the grand total.
- **Descriptions:** one line per entry, exactly as typed, in date and time order within a category. No dates. Repeated descriptions are not merged. Line breaks inside a description become single spaces.
- **Billable only:** non-billable entries are left out of the summary and its totals.
- **Month assignment:** each stored piece counts in the month of its own local date. An entry that crosses midnight at a month end contributes to both months.
- **Empty month:** the text says that no billable time was recorded.
- **Language:** all labels come from the message catalog (`summary.*`, `month.*`), so the text can be translated.
- **Per-member breakdown:** not in R1.
