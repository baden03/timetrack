import { useCallback, useEffect, useState } from "react";
import { call, type ProjectOverview, type Rec, type TimeEntry, type TimerState } from "../api";
import type { Data } from "../App";
import { colorOf } from "../colors";
import { ColorDot } from "../components/Color";
import { EntryForm } from "../components/EntryForm";
import { EntryList } from "../components/EntryList";
import { ErrorLine, errorText } from "../components/Errors";
import { t } from "../i18n";
import { addDays, formatHhmm, todayIn } from "../time";

/** One client project: totals, the entries, and a form with the project fixed. */
export function ProjectPage({
  data,
  projectId,
  version,
  onBack,
  onChanged,
}: {
  data: Data;
  projectId: string;
  version: number;
  onBack: () => void;
  onChanged: () => void;
}) {
  const settings = data.state.settings.payload;
  const project = data.projects.find((p) => p.id === projectId);
  const client = data.clients.find((c) => c.id === project?.payload.client_id);
  const today = todayIn(settings.timezone);
  const [windowDays, setWindowDays] = useState(settings.entry_window_days);
  const [entries, setEntries] = useState<Rec<TimeEntry>[]>([]);
  const [overview, setOverview] = useState<ProjectOverview | null>(null);
  const [view, setView] = useState<"day" | "week">("day");
  const [timerRunning, setTimerRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const from = addDays(today, -windowDays);
      const all = await call<Rec<TimeEntry>[]>("list_entries", { from });
      setEntries(all.filter((e) => e.payload.project_id === projectId));
      setOverview(await call<ProjectOverview>("project_overview", { project_id: projectId }));
      setTimerRunning((await call<TimerState>("timer_state")).timer !== null);
      setError(null);
    } catch (e) {
      setError(errorText(e));
    }
  }, [today, windowDays, projectId]);

  useEffect(() => {
    void load();
  }, [load, version]);

  if (!project) return null;
  const changed = async () => {
    await load();
    onChanged();
  };
  const tiles: [string, number | undefined][] = [
    ["ui.project.today", overview?.today_minutes],
    ["ui.project.week", overview?.week_minutes],
    ["ui.project.month", overview?.month_minutes],
    ["ui.project.total", overview?.total_minutes],
  ];

  return (
    <div>
      <button className="link back" onClick={onBack}>
        ← {t("ui.nav.dashboard")}
      </button>
      <div className="row title">
        <h1>
          <ColorDot color={colorOf(project.payload, client?.payload)} />
          {client?.payload.name} / {project.payload.name}
        </h1>
        <button
          disabled={timerRunning}
          title={timerRunning ? t("ui.dashboard.timer_busy") : undefined}
          onClick={async () => {
            try {
              await call("start_timer", { timer: { project_id: projectId } });
              await changed();
            } catch (e) {
              setError(errorText(e));
            }
          }}
        >
          {t("ui.timer.start")}
        </button>
      </div>
      {project.payload.description && <p className="muted">{project.payload.description}</p>}
      <ErrorLine message={error} />
      <dl className="tiles">
        {tiles.map(([key, minutes]) => (
          <div key={key}>
            <dt>{t(key)}</dt>
            <dd>{minutes === undefined ? "–" : formatHhmm(minutes)}</dd>
          </div>
        ))}
      </dl>
      <EntryForm data={data} today={today} onSaved={changed} fixedProjectId={projectId} />
      <div className="row">
        <label className="check">
          <input type="radio" name="project-view" checked={view === "day"} onChange={() => setView("day")} />
          {t("ui.entries.view_day")}
        </label>
        <label className="check">
          <input type="radio" name="project-view" checked={view === "week"} onChange={() => setView("week")} />
          {t("ui.entries.view_week")}
        </label>
      </div>
      <EntryList data={data} entries={entries} view={view} onChanged={changed} hideProject />
      <button className="link" onClick={() => setWindowDays(windowDays + settings.entry_window_days)}>
        {t("ui.entries.older", { count: windowDays })}
      </button>
    </div>
  );
}
