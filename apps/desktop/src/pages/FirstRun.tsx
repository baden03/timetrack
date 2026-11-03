import { useState } from "react";
import { call } from "../api";
import { t } from "../i18n";
import { useBackup } from "../components/Backup";
import { ErrorLine } from "../components/Errors";

export function FirstRun({ onDone }: { onDone: () => Promise<void> }) {
  const [busy, setBusy] = useState(false);
  const backup = useBackup(onDone);
  return (
    <main className="first-run">
      <h1>{t("ui.firstrun.title")}</h1>
      <p>{t("ui.firstrun.body")}</p>
      <button
        autoFocus
        disabled={busy}
        onClick={async () => {
          setBusy(true);
          await call("finish_first_run");
          await onDone();
        }}
      >
        {t("ui.firstrun.standalone")}
      </button>
      <p>
        <button disabled={busy || backup.busy} onClick={() => void backup.restoreFromFile()}>
          {t("ui.firstrun.restore")}
        </button>
      </p>
      {backup.dialog}
      <ErrorLine message={backup.error} />
      <p className="muted">{t("ui.firstrun.hub_later")}</p>
    </main>
  );
}
