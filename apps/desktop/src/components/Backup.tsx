import { useState } from "react";
import { call } from "../api";
import type { Data } from "../App";
import { chooseBackupOpenPath, chooseBackupSavePath } from "../files";
import { t } from "../i18n";
import { displayDate, displayTime } from "../time";
import { useDialogs } from "./Dialogs";
import { errorText } from "./Errors";

interface BackupInfo {
  schema_version: number;
  records: number;
  history_entries: number;
}

/** Backing up to a file and restoring from one (concept §16.2). */
export function useBackup(reload: () => Promise<void>, settings?: Data["state"]["settings"]["payload"]) {
  const dialogs = useDialogs();
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const run = async (fn: () => Promise<string | null>) => {
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      setMessage(await fn());
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const backUpNow = () =>
    run(async () => {
      const stamp = new Date().toISOString().slice(0, 10);
      const path = await chooseBackupSavePath(`timetrack-backup-${stamp}.sqlite`, dialogs.prompt);
      if (!path) return null;
      const info = await call<BackupInfo>("backup", { path });
      await reload();
      return t("ui.backup.done", { records: info.records, path });
    });

  const restoreFromFile = () =>
    run(async () => {
      const path = await chooseBackupOpenPath(dialogs.prompt);
      if (!path) return null;
      const info = await call<BackupInfo>("inspect_backup", { path });
      const ok = await dialogs.confirm(
        t("ui.backup.restore_confirm", { records: info.records, history: info.history_entries }),
      );
      if (!ok) return null;
      await call("restore", { path });
      await reload();
      return t("ui.backup.restored");
    });

  const describe = (iso: string | null | undefined) => {
    if (!iso) return t("ui.backup.never");
    const zone = settings?.timezone ?? "UTC";
    const day = new Intl.DateTimeFormat("en-CA", { timeZone: zone }).format(new Date(iso));
    return `${displayDate(day, settings?.date_format ?? "DD.MM.YYYY")} ${displayTime(iso, zone, settings?.clock_24h ?? true)}`;
  };

  return { backUpNow, restoreFromFile, describe, message, error, busy, dialog: dialogs.dialog };
}
