import { useCallback, useEffect, useState, type ReactNode } from "react";
import { colorOf } from "../colors";
import { ColorDot } from "./Color";
import { call, inTauri, type Category, type Rec, type SavedEntry, type TimerState } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "./Errors";
import { t, tError } from "../i18n";
import { clockIn, displayDate, formatElapsed, parseClock } from "../time";
import { useDialogs } from "./Dialogs";
import { TimeInput } from "./FormatInputs";

/** The persistent timer indicator (concept §9) and the stale-timer prompt. */
export function TimerBar({
  data,
  version,
  showIdle,
  onChanged,
}: {
  data: Data;
  version: number;
  /** The start controls are hidden where the page offers its own. */
  showIdle: boolean;
  onChanged: () => void;
}) {
  const [state, setState] = useState<TimerState>({
    timer: null,
    stale: false,
    suggested_category_id: null,
  });
  // What is typed in the note field and not yet entered.
  const [draft, setDraft] = useState("");
  const timerId = state.timer?.id;
  const savedDraft = state.timer?.payload.description ?? "";
  useEffect(() => {
    setDraft(savedDraft);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [timerId]);
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

  // Refresh when something else (a dashboard card) started or changed it.
  useEffect(() => {
    void refresh();
  }, [refresh, version]);

  // Tick on the whole seconds of the elapsed time, counted from the same
  // stored start as the menu bar, so the two change at the same moment.
  const startedAt = state.timer
    ? new Date(state.timer.payload.exact_start_at ?? state.timer.payload.start_at).getTime()
    : null;
  useEffect(() => {
    if (startedAt === null) return;
    let handle: ReturnType<typeof setTimeout>;
    const schedule = () => {
      setNow(Date.now());
      const intoSecond = (Date.now() - startedAt) % 1000;
      handle = setTimeout(schedule, Math.max(20, 1000 - ((intoSecond + 1000) % 1000)));
    };
    schedule();
    return () => clearTimeout(handle);
  }, [startedAt]);

  // The global shortcut and the tray menu live in the shell; they ask the
  // window to open the stop prompt. (Changes to the timer reach the whole
  // window through App, which reloads everything when the shell announces one.)
  useEffect(() => {
    if (!inTauri) return;
    const unlisten: Array<() => void> = [];
    void import("@tauri-apps/api/event").then(async ({ listen }) => {
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
  if (!timer) return showIdle ? <IdleBar data={data} error={error} run={run} /> : null;

  const p = timer.payload;
  const project = data.projects.find((x) => x.id === p.project_id);
  const client = data.clients.find((c) => c.id === project?.payload.client_id);
  const category = data.categories.find((c) => c.id === p.category_id);
  const label = (
    <>
      <ColorDot color={colorOf(project?.payload, client?.payload)} />
      {client?.payload.name ?? ""} / {project?.payload.name ?? ""}
      {category ? ` — ${category.payload.name}` : ""}
    </>
  );
  const projectCategories = data.categories.filter(
    (c) => c.id === p.category_id || (c.payload.status === "active" && project?.payload.category_ids.includes(c.id)),
  );

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
        clock24h={settings.clock_24h}
        initialDescription={draft}
        notes={p.notes ?? []}
        categories={projectCategories}
        initialCategory={p.category_id ?? state.suggested_category_id ?? ""}
        error={error}
        onStop={(description, endTime, categoryId) =>
          run(async () => {
            await call<SavedEntry>("stop_timer", {
              stop: { description, end_time: endTime, category_id: categoryId || null },
            });
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

  const addNote = async () => {
    const text = draft.trim();
    if (!text) return;
    // Cleared at once, so the next note can be typed straight away; given back
    // if the note could not be added.
    setDraft("");
    try {
      await call("add_timer_note", { text });
      setError(null);
      await refresh();
      onChanged();
    } catch (e) {
      setDraft(text);
      setError(errorText(e));
    }
  };

  return (
    <div className="timer-bar running" role="status">
      <span className="dot" aria-hidden="true">●</span>
      <strong className="what">{label}</strong>
      <span className="elapsed" aria-label={t("ui.timer.elapsed")}>
        {formatElapsed(now - new Date(p.exact_start_at ?? p.start_at).getTime())}
      </span>
      <input
        aria-label={t("ui.timer.note_label")}
        placeholder={t("ui.timer.note_placeholder")}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            void addNote();
          }
        }}
        onBlur={() => {
          // Keep what was typed, in case the app is closed before it is entered.
          if (draft !== p.description) void run(() => call("update_timer", { description: draft }));
        }}
      />
      <button onClick={() => setStopping(true)}>{t("ui.timer.stop")}</button>
      {(p.notes ?? []).length > 0 && (
        <ul className="notes" aria-label={t("ui.timer.notes")}>
          {p.notes.map((note, i) => (
            <li key={`${i}-${note}`}>
              <span>{note}</span>
              <button
                className="link"
                aria-label={t("ui.timer.remove_note", { note })}
                onClick={() => void run(() => call("remove_timer_note", { index: i }))}
              >
                ✕
              </button>
            </li>
          ))}
        </ul>
      )}
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
  const projects = data.projects.filter(
    (p) =>
      p.payload.status === "active" &&
      data.clients.find((c) => c.id === p.payload.client_id)?.payload.status === "active",
  );
  const [projectId, setProjectId] = useState(projects[0]?.id ?? "");
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
      <button onClick={() => run(() => call("start_timer", { timer: { project_id: projectId } }))}>
        {t("ui.timer.start")}
      </button>
      <ErrorLine message={error} />
    </div>
  );
}

function StopForm(props: {
  label: ReactNode;
  notes: string[];
  stale: boolean;
  startedOn: string;
  startedAt: string;
  clock24h: boolean;
  initialDescription: string;
  categories: Rec<Category>[];
  initialCategory: string;
  error: string | null;
  onStop: (description: string, endTime: string | undefined, categoryId: string) => Promise<void>;
  onDiscard: () => Promise<void>;
  onCancel?: () => void;
}) {
  const [description, setDescription] = useState(props.initialDescription);
  const [endTime, setEndTime] = useState("");
  const [categoryId, setCategoryId] = useState(props.initialCategory);
  const [localError, setLocalError] = useState<string | null>(null);
  const dialogs = useDialogs();
  return (
    <>
    {dialogs.dialog}
    <form
      className="timer-bar stale card"
      onSubmit={(e) => {
        e.preventDefault();
        if (props.stale) {
          const parsed = parseClock(endTime);
          if (parsed === null) return setLocalError(tError("invalid_date_time"));
          void props.onStop(description, parsed, categoryId);
        } else {
          void props.onStop(description, undefined, categoryId);
        }
      }}
      onKeyDown={(e) => e.key === "Escape" && props.onCancel?.()}
    >
      <strong className="what">{props.label}</strong>
      {props.notes.length > 0 && (
        <ul className="notes" aria-label={t("ui.timer.notes")}>
          {props.notes.map((note, i) => (
            <li key={`${i}-${note}`}>{note}</li>
          ))}
        </ul>
      )}
      {props.stale && (
        <p role="alert">{t("ui.timer.stale", { date: props.startedOn, time: props.startedAt })}</p>
      )}
      <div className="row">
        {props.stale && (
          <label>
            {t("ui.timer.real_stop")}
            <TimeInput required value={endTime} clock24h={props.clock24h} onChange={(v) => { setLocalError(null); setEndTime(v); }} />
          </label>
        )}
        <label>
          {t("ui.entries.category")}
          <select value={categoryId} onChange={(e) => setCategoryId(e.target.value)}>
            <option value="">{t("ui.entries.no_category")}</option>
            {props.categories.map((c) => (
              <option key={c.id} value={c.id}>
                {c.payload.name}
              </option>
            ))}
          </select>
        </label>
        <label className="grow">
          {props.notes.length > 0 ? t("ui.timer.last_note") : t("ui.entries.description")}
          <input
            autoFocus
            required={props.notes.length === 0}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </label>
        <button type="submit">{props.stale ? t("ui.timer.save_entry") : t("ui.timer.confirm_stop")}</button>
        <button
          type="button"
          className="danger"
          onClick={async () => {
            if (await dialogs.confirm(t("ui.timer.discard_confirm"))) void props.onDiscard();
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
      <ErrorLine message={localError ?? props.error} />
    </form>
    </>
  );
}
