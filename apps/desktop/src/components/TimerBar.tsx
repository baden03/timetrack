import { useCallback, useEffect, useState } from "react";
import { call, inTauri, type SavedEntry, type TimerState } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "./Errors";
import { t } from "../i18n";
import { clockIn, displayDate, formatElapsed } from "../time";

/** The persistent timer indicator (concept §9) and the stale-timer prompt. */
export function TimerBar({ data, onChanged }: { data: Data; onChanged: () => void }) {
  const [state, setState] = useState<TimerState>({ timer: null, stale: false });
  const [now, setNow] = useState(Date.now());
  const [error, setError] = useState<string | null>(null);
  const [stopping, setStopping] = useState(false);
  const settings = data.state.settings.payload;

  const refresh = useCallback(async () => {
    try {
      setState(await call<TimerState>("timer_state"));
    } catch (e) {
      setError(errorText(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const tick = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(tick);
  }, [refresh]);

  // The menu bar and the global shortcut live in the shell and tell the
  // window when to refresh or to open the stop prompt.
  useEffect(() => {
    if (!inTauri) return;
    const unlisten: Array<() => void> = [];
    void import("@tauri-apps/api/event").then(async ({ listen }) => {
      unlisten.push(await listen("timer-changed", () => void refresh()));
      unlisten.push(
        await listen("timer-stop-request", () => {
          void refresh().then(() => setStopping(true));
        }),
      );
    });
    return () => unlisten.forEach((u) => u());
  }, [refresh]);

  const run = async (fn: () => Promise<unknown>) => {
    try {
      await fn();
      setError(null);
      await refresh();
      onChanged();
    } catch (e) {
      setError(errorText(e));
    }
  };

  const timer = state.timer;
  if (!timer) return <IdleBar data={data} error={error} run={run} />;

  const p = timer.payload;
  const project = data.projects.find((x) => x.id === p.project_id);
  const client = data.clients.find((c) => c.id === project?.payload.client_id);
  const category = data.categories.find((c) => c.id === p.category_id);
  const label = `${client?.payload.name ?? ""} / ${project?.payload.name ?? ""} — ${category?.payload.name ?? ""}`;

  if (state.stale || stopping) {
    return (
      <StopForm
        key={timer.id + String(state.stale)}
        label={label}
        stale={state.stale}
        startedOn={displayDate(
          new Intl.DateTimeFormat("en-CA", { timeZone: p.timezone }).format(new Date(p.start_at)),
          settings.date_format,
        )}
        startedAt={clockIn(p.start_at, p.timezone)}
        initialDescription={p.description}
        error={error}
        onStop={(description, endTime) =>
          run(async () => {
            await call<SavedEntry>("stop_timer", { stop: { description, end_time: endTime } });
            setStopping(false);
          })
        }
        onDiscard={() =>
          run(async () => {
            await call("discard_timer");
            setStopping(false);
          })
        }
        onCancel={state.stale ? undefined : () => setStopping(false)}
      />
    );
  }

  return (
    <div className="timer-bar running" role="status">
      <span className="dot" aria-hidden="true">●</span>
      <strong>{label}</strong>
      <span className="elapsed" aria-label={t("ui.timer.elapsed")}>
        {formatElapsed(now - new Date(p.start_at).getTime())}
      </span>
      <input
        aria-label={t("ui.entries.description")}
        placeholder={t("ui.entries.description")}
        defaultValue={p.description}
        onBlur={(e) => {
          if (e.target.value !== p.description)
            void run(() => call("update_timer", { description: e.target.value }));
        }}
      />
      <button onClick={() => setStopping(true)}>{t("ui.timer.stop")}</button>
      <ErrorLine message={error} />
    </div>
  );
}

function IdleBar({
  data,
  error,
  run,
}: {
  data: Data;
  error: string | null;
  run: (fn: () => Promise<unknown>) => Promise<void>;
}) {
  const projects = data.projects.filter((p) => p.payload.status === "active");
  const [projectId, setProjectId] = useState(projects[0]?.id ?? "");
  const project = projects.find((p) => p.id === projectId);
  const categories = data.categories.filter(
    (c) => c.payload.status === "active" && project?.payload.category_ids.includes(c.id),
  );
  const [categoryId, setCategoryId] = useState("");
  const chosen = categories.some((c) => c.id === categoryId) ? categoryId : categories[0]?.id ?? "";
  if (projects.length === 0) return null;
  return (
    <div className="timer-bar">
      <label>
        {t("ui.entries.project")}
        <select value={projectId} onChange={(e) => setProjectId(e.target.value)}>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>
              {data.clients.find((c) => c.id === p.payload.client_id)?.payload.name} / {p.payload.name}
            </option>
          ))}
        </select>
      </label>
      <label>
        {t("ui.entries.category")}
        <select value={chosen} onChange={(e) => setCategoryId(e.target.value)}>
          {categories.map((c) => (
            <option key={c.id} value={c.id}>
              {c.payload.name}
            </option>
          ))}
        </select>
      </label>
      <button
        disabled={!chosen}
        onClick={() =>
          run(() => call("start_timer", { timer: { project_id: projectId, category_id: chosen } }))
        }
      >
        {t("ui.timer.start")}
      </button>
      <ErrorLine message={error} />
    </div>
  );
}

function StopForm(props: {
  label: string;
  stale: boolean;
  startedOn: string;
  startedAt: string;
  initialDescription: string;
  error: string | null;
  onStop: (description: string, endTime?: string) => Promise<void>;
  onDiscard: () => Promise<void>;
  onCancel?: () => void;
}) {
  const [description, setDescription] = useState(props.initialDescription);
  const [endTime, setEndTime] = useState("");
  return (
    <form
      className="timer-bar stale card"
      onSubmit={(e) => {
        e.preventDefault();
        void props.onStop(description, props.stale ? endTime : undefined);
      }}
      onKeyDown={(e) => e.key === "Escape" && props.onCancel?.()}
    >
      <strong>{props.label}</strong>
      {props.stale && (
        <p role="alert">{t("ui.timer.stale", { date: props.startedOn, time: props.startedAt })}</p>
      )}
      <div className="row">
        {props.stale && (
          <label>
            {t("ui.timer.real_stop")}
            <input type="time" required value={endTime} onChange={(e) => setEndTime(e.target.value)} />
          </label>
        )}
        <label className="grow">
          {t("ui.entries.description")}
          <input
            autoFocus
            required
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </label>
        <button type="submit">{props.stale ? t("ui.timer.save_entry") : t("ui.timer.confirm_stop")}</button>
        <button
          type="button"
          className="danger"
          onClick={() => {
            if (window.confirm(t("ui.timer.discard_confirm"))) void props.onDiscard();
          }}
        >
          {t("ui.timer.discard")}
        </button>
        {props.onCancel && (
          <button type="button" onClick={props.onCancel}>
            {t("ui.cancel")}
          </button>
        )}
      </div>
      <ErrorLine message={props.error} />
    </form>
  );
}
