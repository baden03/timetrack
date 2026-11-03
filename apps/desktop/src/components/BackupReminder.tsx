import { useState } from "react";
import type { Data } from "../App";
import { t } from "../i18n";
import { useBackup } from "./Backup";
import { ErrorLine } from "./Errors";

/** A quiet reminder when no backup has been made for a while (concept §16.2). */
export function BackupReminder({ data, reload }: { data: Data; reload: () => Promise<void> }) {
  const settings = data.state.settings.payload;
  const backup = useBackup(reload, settings);
  const [dismissed, setDismissed] = useState(false);
  const days = settings.backup_reminder_days;
  const last = data.state.last_backup_at;
  const ageDays = last ? (Date.now() - new Date(last).getTime()) / 86_400_000 : Infinity;
  if (dismissed || days === 0 || ageDays < days) return null;
  return (
    <div className="banner" role="status">
      {backup.dialog}
      <span>{last ? t("ui.backup.reminder_old", { when: backup.describe(last) }) : t("ui.backup.reminder_never")}</span>
      <button disabled={backup.busy} onClick={() => void backup.backUpNow()}>
        {t("ui.backup.now")}
      </button>
      <button onClick={() => setDismissed(true)}>{t("ui.backup.later")}</button>
      <ErrorLine message={backup.error} />
    </div>
  );
}
