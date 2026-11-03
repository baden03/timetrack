# Installing TimeTrack on a Mac (release 1 test build)

This build is for Baden, Doug and Steffen to try. It works on one Mac with no account and no network connection. Each person's data stays on their own Mac; sharing between people arrives in release 2.

## Install

1. Unzip `TimeTrack.zip` and drag **TimeTrack** into the **Applications** folder.
2. The first time, macOS will refuse to open it ("cannot be opened because the developer cannot be verified"). This build is signed only on the build machine and is not notarized by Apple. Do one of these once:
   - In Finder, **right-click TimeTrack, choose Open, then Open** in the dialog; or
   - open **System Settings, Privacy & Security**, scroll to the message about TimeTrack and choose **Open Anyway**; or
   - in Terminal: `xattr -dr com.apple.quarantine /Applications/TimeTrack.app`
3. Choose "Use TimeTrack on this Mac only" on the first screen.

Needs macOS 12 or newer.

## Where your data lives

`~/Library/Application Support/de.twinpictures.timetrack/timetrack.sqlite` is the whole database, including the change history. Back it up from the app (below) rather than copying it while the app runs.

## Backups

**Settings, Back up now** writes one file wherever you choose. A reminder appears on every page when the last backup is older than the number of days set in Settings (default 7; 0 turns it off). **Restore from backup** replaces everything on this Mac with that file after asking you to confirm. The current data is copied into the app's `backups` folder first, next to the database, so a mistaken restore can be undone. On a new Mac, the first screen also offers **Restore from a backup**.

## Menu bar and shortcut

- While a timer runs, its elapsed time is shown in the menu bar.
- **Cmd+Shift+T** starts a timer on the project of your last entry; if one is running it opens the stop prompt.
- The menu-bar icon has **Show TimeTrack**, **Start / Stop timer** and **Quit TimeTrack**.
- Closing the window keeps the app (and the timer) running; **Quit** from the menu-bar icon ends it. Clicking the Dock icon shows the window again.

## What we would like feedback on

- Recording a real working day: start and end entry, the timer, duration only.
- The monthly summary (Summary page, Copy as text): does it match your own figures, and is the text what you need to write the invoice?
- Anything slow, confusing or missing, and whether the keyboard alone is enough to record and correct time.

Please note what you were doing and what you expected. Screenshots help.

## Updating and removing

Replace the app in Applications with the new one. Your data is kept. If a new version changes the database, the old file is first copied next to it as `timetrack.sqlite.pre-vN.bak`. To remove TimeTrack, delete the app; delete the folder above too if you also want to erase the data.

## Privacy

The app makes no network connections and sends nothing anywhere. A check in the repository (`scripts/no-network.sh`) fails the build if a networking library or an outside address is ever added.
