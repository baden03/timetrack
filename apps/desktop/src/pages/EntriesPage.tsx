import { useCallback, useEffect, useState } from "react";
import { call, type Rec, type TimeEntry } from "../api";
import type { Data } from "../App";
import { EntryForm } from "../components/EntryForm";
import { EntryList } from "../components/EntryList";
import { ErrorLine, errorText } from "../components/Errors";
import { t } from "../i18n";
import { addDays, todayIn } from "../time";

export function EntriesPage({ data, version }: { data: Data; version: number }) {
  const settings = data.state.settings.payload;
  const [windowDays, setWindowDays] = useState(settings.entry_window_days);
  const [entries, setEntries] = useState<Rec<TimeEntry>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<"day" | "week">("day");
  const today = todayIn(settings.timezone);

  const load = useCallback(async () => {
    try {
      const from = addDays(today, -windowDays);
      setEntries(await call<Rec<TimeEntry>[]>("list_entries", { from }));
      setError(null);
    } catch (e) {
      setError(errorText(e));
    }
  }, [today, windowDays, version]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div>
      <h1>{t("ui.entries.title")}</h1>
      <ErrorLine message={error} />
      <EntryForm data={data} today={today} onSaved={load} />
      <div className="row">
        <label className="check">
          <input type="radio" name="entries-view" checked={view === "day"} onChange={() => setView("day")} />
          {t("ui.entries.view_day")}
        </label>
        <label className="check">
          <input type="radio" name="entries-view" checked={view === "week"} onChange={() => setView("week")} />
          {t("ui.entries.view_week")}
        </label>
      </div>
      <EntryList data={data} entries={entries} view={view} onChanged={load} />
      <button className="link" onClick={() => setWindowDays(windowDays + settings.entry_window_days)}>
        {t("ui.entries.older", { days: windowDays })}
      </button>
    </div>
  );
}
