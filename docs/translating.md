# Translating TimeTrack into your language

TimeTrack keeps every word the user sees in a catalog file, one file per language. Adding a language means writing one text file. You do not need to be a programmer for the translation itself; registering the language takes two short lines of code, which a maintainer can add for you.

English (`en`) is the base and German (`de`, informal "du") is the first translation. Use `i18n/de.json` as a worked example next to this guide.

## Two ways to contribute

- **Translation only.** Copy `i18n/en.json` to `i18n/<code>.json` (for example `fr.json`), translate it, and open a pull request. This works in the GitHub web editor. A maintainer registers the language and runs the tests.
- **Translation and registration.** Do the same, then follow "Register the language" and "Test it" below so the pull request is complete.

Language codes are the two-letter ISO 639-1 code (`fr`, `es`, `it`, `nl`). Regional variants are not supported yet.

## The file

`i18n/en.json` is a flat JSON object. Each line is `"key": "text"`:

```json
"ui.entries.add": "Add entry",
"ui.entries.older.other": "Showing the last {count} days. Show older entries",
"error.name_required": "A name is required.",
"month.3": "March",
```

**Translate the text on the right. Never change the key on the left.** Keep the keys in the same order as English, and keep every key: a translation file must contain exactly the keys that `en.json` has.

### What the key prefixes mean

| Prefix | Where it appears | Notes |
|---|---|---|
| `ui.*` | Buttons, labels, headings, hints, confirmation questions | Keep buttons short; text must fit |
| `error.*` | Messages shown when something is refused | Say what is wrong and, where it helps, what to do |
| `month.1` to `month.12` | Month names | Used in the monthly summary text |
| `summary.*` | The monthly summary text people copy into invoices | Be careful and consistent; see below |
| `tray.*` | The menu shown from the menu-bar icon | Very short |
| `ui.field.*` | Field names in the change history | |

### Placeholders

Text in curly braces, such as `{name}`, `{date}` or `{total}`, is replaced by the program. Keep every placeholder exactly as written and keep the same set in your translation. You may move them to fit your grammar:

```
en: Week of {start} to {end}
de: Woche vom {start} bis {end}
```

A test fails if a placeholder is missing, added or misspelled.

### Plurals

Messages with a number come in two forms, chosen by the number `{count}`:

```json
"ui.search.total.one":   "{count} entry, {total}",
"ui.search.total.other": "{count} entries, {total}"
```

`.one` is used when the number is exactly 1, `.other` for every other number, including 0. Translate both. If your language does not distinguish them, write the same text in both.

**Limit:** only these two forms exist so far. Languages that need more (for example Polish, Russian, Arabic or Welsh) cannot yet be written exactly. Please open an issue; the program needs a small extension for them.

### Do not translate

- The product name **TimeTrack**.
- User data such as client names or descriptions (the program never translates those).
- The sample values inside `{braces}`.
- The key names.

### Style

- Choose one way of addressing the user and use it everywhere, formal or informal. State your choice in the pull request. German uses the informal form.
- Use your language's normal typographic quotes and spacing. The ellipsis `…` after a button name means "this opens a dialog"; keep it.
- Keep words for the same thing the same everywhere. A small glossary to agree on first:

  | English | Means |
  |---|---|
  | client | the customer a project is for |
  | project | a piece of work for a client |
  | category | the type of work (Development, Meetings) |
  | entry | one recorded stretch of time |
  | timer | the running clock |
  | billable | counts towards the invoice |
  | summary | the monthly text of totals and descriptions |
  | backup / restore | a copy of all data in one file / loading it back |
  | archive | hide without deleting |

- The **summary** text (`summary.*` and `month.*`) is what people copy into invoice tools or send to a translator or an AI. Prefer plain, unambiguous wording over clever wording.

## Register the language

Two places, one line each.

**1. `crates/core/src/i18n.rs`:** add the language to the `CATALOGS` list, and raise the number in its type:

```rust
const CATALOGS: [(&str, &str); 3] = [
    ("en", include_str!("../../../i18n/en.json")),
    ("de", include_str!("../../../i18n/de.json")),
    ("fr", include_str!("../../../i18n/fr.json")),
];
```

**2. `apps/desktop/src/i18n.ts`:** import the file and add it to `LANGUAGES`, with the language's name written in that language (this is what the Settings menu shows):

```ts
import fr from "../../../i18n/fr.json";
// ...
{ code: "fr", name: "Français", messages: fr },
```

The Settings page, the monthly summary, the error messages and the menu-bar menu all follow the language chosen in Settings. Nothing else needs changing.

## Test it

Run the whole check (it needs the tools in [DEVELOPMENT.md](../DEVELOPMENT.md)):

```bash
scripts/check.sh
```

or just the translation tests:

```bash
cargo test -p timetrack-core i18n
cd apps/desktop && pnpm test
```

The tests check, for every language except English:

- it has exactly the keys English has, no more and no fewer;
- no text is empty;
- each text uses the same `{placeholders}` as the English one;
- the file is valid JSON that the program can load.

Then look at it in the app. Use a throwaway data folder so your own data is untouched:

```bash
cd apps/desktop && TT_DATA_DIR=/tmp/tt-translate pnpm tauri dev
```

Open **Settings**, choose your language in the language list, and press **Save**. The interface changes at once. Click through every page: Dashboard, Time, Search, Summary, Setup and Settings. Look for text that is cut off, buttons that no longer fit, and sentences that read oddly with real names filled in. Try a client called, say, "Müller & Söhne" and an entry that crosses midnight to see the longer messages. Start a timer and check the menu-bar menu as well.

## What the language does not change

- **Dates and the clock** follow their own settings (`DD.MM.YYYY`, `YYYY-MM-DD`, `MM/DD/YYYY`; 24-hour or 12-hour), because people mix them: a German interface with a US date format is allowed.
- **Durations** are always `h:mm`, such as `126:30`.
- **Left-to-right layout** is assumed. The styling avoids fixed left and right where it can, but right-to-left languages have not been tried. If you translate into one, please tell us what breaks.

## Keeping a translation current

When a new message is added to `en.json`, the tests fail for every other language until it has been translated. That is intentional: a missing message would otherwise appear in English in the middle of a translated screen. (Inside the app a missing message does fall back to English, so nothing breaks for a user while it is waiting.)

## Pull request checklist

- [ ] `i18n/<code>.json` has the same keys, in the same order, as `en.json`
- [ ] Both plural forms are translated where they exist
- [ ] `scripts/check.sh` passes (or a maintainer will run it)
- [ ] You looked at every page in the app in your language
- [ ] The pull request says whether you used the formal or informal form, and who the translation was reviewed by, if anyone
