# TimeTrack quick start

A walk through setting up a new stand-alone installation and recording your first hours. It takes about ten minutes. Everything stays on your Mac: there is no account and no network connection.

TimeTrack keeps time for **clients** and their **projects**. You record hours against a project, and at the end of the month copy a summary, per client, project and category, into your invoice.

## 1. Install

1. Download `TimeTrack-<version>-macos-arm64.zip` from the [Releases page](https://github.com/baden03/timetrack/releases) (Apple Silicon Macs, macOS 12 or newer).
2. Unzip it and drag **TimeTrack** into **Applications**.
3. The first time, macOS says it cannot verify the developer. Right-click **TimeTrack** in Applications, choose **Open**, then **Open** again. You only do this once. Other ways, if that does not work, are in [install-macos.md](install-macos.md).

## 2. First launch

The first screen asks how to start:

- **Use TimeTrack on this Mac only**: choose this for a new installation.
- **Restore from a backup**: choose this instead if you are moving to a new Mac and have a backup file (see [section 8](#8-back-up-your-data)).

After that you land on the **Dashboard**, which is empty. The menu on the left has **Dashboard, Time, Search, Summary, Setup, Settings** and **About**.

## 3. Set up your work

Open **Setup**. You need a client and a project; categories are optional.

1. **Add a client.** Type its name under **Clients** and press **Add**. Use **Color** on the client to pick a color from the palette (or none). Colors make cards and lists easier to find; they adjust to the light and dark theme on their own.
2. **Add categories (optional).** Under **Categories**, add the kinds of work you want to tell apart, such as *Development* and *Meetings*. You can skip this and record time with no category.
3. **Add a project.** Under **Projects** choose **Add project**, pick the client, give it a name, decide whether it is **billable by default**, and tick the categories that apply to it. A project can have its own color; otherwise it uses its client's.

Nothing is final: you can rename, recolor, archive and restore clients, projects and categories later. **Archive** hides something without losing its time; **Delete** is only possible while nothing uses it.

## 4. Check your settings

Open **Settings** and look at these once:

| Setting | What it does |
|---|---|
| Time zone for new entries | Starts as your Mac's zone. Each entry keeps the zone it was recorded in. |
| Date format, 24-hour clock | How dates and times are shown and typed. |
| Theme | Match the system, light or dark. |
| Language | English or Deutsch. |
| Days of entries shown | How far back the Time page lists by default (60). |
| Backup reminder | After how many days without a backup you are reminded (7; 0 turns it off). |

Press **Save**.

## 5. Record your first time

There are three ways. Each ends as an entry with a project, a description, optionally a category and tags, and a *billable* flag.

### a) Start and end

On the **Time** page choose the project, enter the **From** and **To** times and a description, then click **Add entry**.

- **Description a line at a time:** press **Enter** to add a line to the list under the field, then type the next one. Press **Enter** on an empty line to save the entry (or **Cmd+Enter** at any point). One line stays plain text; several become bullet points.
- Type times freely: `2130`, `930` and `9:30` all work, and the field tidies itself when you leave it. The same goes for dates in the format you chose.
- If the end is earlier than the start, the entry runs past midnight and is split into two linked parts, one for each day.
- The end may be up to 15 minutes in the future, so you can round 14:27 up to 14:30.

### b) Duration only

Change **Record by** to **Duration** and type the time spent: `1:30`, `:45`, `.75`, `1.5` or `1h30`. A bare number up to 24 is hours, a larger one is minutes (`45` is 0:45).

### c) The timer

On the **Dashboard**, press **Start timer** on a project's card. A timer bar appears at the top, with the elapsed time; it also shows in the menu bar.

- **Add notes as you go:** type a line and press **Enter**. Each one is listed under the timer.
- Press **Stop**. Confirm the category (it suggests the one you used last on the project), type a last note if you like, and press **Stop and save**. Several notes become bullet points of one entry.
- If you left the timer running overnight, TimeTrack asks when you really stopped, or lets you discard it.
- A timer needs at least a minute to become an entry.

**Shortcut:** **Cmd+Shift+T** starts a timer on your last project from anywhere. Pressing it while a timer runs brings up the stop prompt. Closing the window does not stop the timer: it keeps running from the menu bar icon, which also has **Show TimeTrack** and **Quit**.

## 6. Look around

- **Dashboard:** one card per project, the most recently used first, each with a start button. Click a card for that project's page: totals for today, this week, this month and all time, its entries, and a form that already knows the project.
- **Time:** all entries by day or week. Double-click a row, or press Enter on it, to edit it; **Enter** saves and **Esc** cancels. **History** on a row shows every change with its before and after.
- **Search:** find entries by text, client, project, category, tag, billable flag or dates; it shows the total of what it found.

## 7. At the end of the month

Open **Summary**, choose the month and press **Copy as text**. You get, per client, project and category, the total hours and a line for each description, for billable time only. Paste it into your invoicing tool, or hand it to an assistant to write and translate the invoice text.

Hours always come from what you recorded, so the category totals add up to the project total.

## 8. Back up your data

Your time exists only on this Mac, so back it up. In **Settings**, press **Back up now…** and save the file somewhere safe (a cloud folder is fine). TimeTrack reminds you when the last backup is older than the number of days you set.

To restore, use **Restore from backup…** in Settings, or the button on the first screen of a fresh installation. Your current data is copied aside first, so a mistaken restore can be undone.

## If something goes wrong

| What you see | What to do |
|---|---|
| macOS refuses to open the app | Right-click, **Open** (section 1). |
| "The timer has run for less than a minute" | Keep it running a little longer, or **Discard timer**. |
| "An entry cannot end more than 15 minutes in the future" | Check the date and the end time. |
| An entry shows the wrong day | Each entry keeps the time zone it was recorded in. Check **Settings, Time zone** for new entries. |
| You deleted or changed something by mistake | Look at the entry's **History**, or restore a backup. |

More: [what was built and what is still planned](plan.md), [translating the interface](translating.md), and the **About** screen for the version and the source code.
