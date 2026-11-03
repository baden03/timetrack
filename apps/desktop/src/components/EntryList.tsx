import { useState } from "react";
import { call, type Rec, type TimeEntry, type Timing } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "./Errors";
import { t, tError } from "../i18n";
import { HistoryPanel } from "./HistoryPanel";
import { colorOf } from "../colors";
import { useDialogs } from "./Dialogs";
import { ColorDot } from "./Color";
import { DescriptionText } from "./DescriptionText";
import { DateInput, TimeInput } from "./FormatInputs";
import {
  addDays,
  clockIn,
  displayDate,
  displayTime,
  formatClock,
  formatHhmm,
  parseClock,
  parseDate,
  parseHhmm,
  weekStart,
} from "../time";

export function EntryList({
  data,
  entries,
  view,
  onChanged,
  hideProject = false,
}: {
  data: Data;
  entries: Rec<TimeEntry>[];
  view: "day" | "week";
  onChanged: () => Promise<void>;
  /** On a project's own page every entry is for that project. */
  hideProject?: boolean;
}) {
  const settings = data.state.settings.payload;
  const [editing, setEditing] = useState<string | null>(null); // record id of the row being edited
  const dialogs = useDialogs();
  const [error, setError] = useState<string | null>(null);
  const [historyOf, setHistoryOf] = useState<string | null>(null); // record id

  if (entries.length === 0) return <p className="muted">{t("ui.entries.empty")}</p>;

  const byDate = new Map<string, Rec<TimeEntry>[]>();
  for (const e of [...entries].sort((a, b) => a.payload.piece_index - b.payload.piece_index)) {
    const list = byDate.get(e.payload.work_date) ?? [];
    list.push(e);
    byDate.set(e.payload.work_date, list);
  }
  const dates = [...byDate.keys()].sort().reverse();

  const remove = async (e: Rec<TimeEntry>) => {
    const text =
      e.payload.piece_count > 1 ? t("ui.entries.delete_group_confirm") : t("ui.entries.delete_confirm");
    if (!(await dialogs.confirm(text))) return;
    try {
      await call("delete_entry", { group: e.payload.group_id });
      await onChanged();
    } catch (err) {
      setError(errorText(err));
    }
  };

  return (
    <div>
      {dialogs.dialog}
      <ErrorLine message={error} />
      {dates.map((date, index) => {
        const rows = byDate.get(date)!;
        const total = rows.reduce((sum, r) => sum + r.payload.minutes, 0);
        const week = weekStart(date);
        const startsWeek = view === "week" && (index === 0 || weekStart(dates[index - 1]) !== week);
        const weekTotal = dates
          .filter((d) => weekStart(d) === week)
          .reduce((sum, d) => sum + byDate.get(d)!.reduce((a, r) => a + r.payload.minutes, 0), 0);
        return (
          <div key={date}>
            {startsWeek && (
              <h2 className="week">
                {t("ui.entries.week_of", {
                  start: displayDate(week, settings.date_format),
                  end: displayDate(addDays(week, 6), settings.date_format),
                })}
                <span className="total">{formatHhmm(weekTotal)}</span>
              </h2>
            )}
          <section className="day">
            <h2>
              {displayDate(date, settings.date_format)}
              <span className="total">{formatHhmm(total)}</span>
            </h2>
            <table>
              <thead>
                <tr>
                  <th scope="col">{t("ui.entries.from")}</th>
                  <th scope="col">{t("ui.entries.to")}</th>
                  <th scope="col">{t("ui.entries.duration")}</th>
                  {!hideProject && <th scope="col">{t("ui.entries.project")}</th>}
                  <th scope="col">{t("ui.entries.category")}</th>
                  <th scope="col">{t("ui.entries.description")}</th>
                  <th scope="col">{t("ui.entries.billable")}</th>
                  <td />
                </tr>
              </thead>
              <tbody>
                {rows
                  .sort((a, b) => (a.payload.start_at ?? "").localeCompare(b.payload.start_at ?? ""))
                  .map((rec) =>
                    editing === rec.id ? (
                      <EditRow
                        key={rec.id}
                        data={data}
                        rec={rec}
                        columns={hideProject ? 7 : 8}
                        onDone={async (changed) => {
                          setEditing(null);
                          if (changed) await onChanged();
                          // Keep the keyboard where it was: on the row just edited.
                          requestAnimationFrame(() =>
                            document.querySelector<HTMLElement>(`[data-record="${rec.id}"]`)?.focus(),
                          );
                        }}
                      />
                    ) : (
                      <ViewRow
                        key={rec.id}
                        data={data}
                        rec={rec}
                        hideProject={hideProject}
                        onEdit={() => setEditing(rec.id)}
                        onDelete={() => remove(rec)}
                        historyOpen={historyOf === rec.id}
                        onHistory={() => setHistoryOf(historyOf === rec.id ? null : rec.id)}
                      />
                    ),
                  )}
              </tbody>
            </table>
          </section>
          </div>
        );
      })}
    </div>
  );
}

function names(data: Data, e: TimeEntry) {
  const project = data.projects.find((p) => p.id === e.project_id);
  const client = data.clients.find((c) => c.id === project?.payload.client_id);
  const category = data.categories.find((c) => c.id === e.category_id);
  return {
    color: colorOf(project?.payload, client?.payload),
    project: `${client?.payload.name ?? ""} / ${project?.payload.name ?? ""}`,
    category: category?.payload.name ?? t("ui.entries.no_category"),
  };
}

function ViewRow({
  data,
  rec,
  hideProject,
  onEdit,
  onDelete,
  historyOpen,
  onHistory,
}: {
  data: Data;
  rec: Rec<TimeEntry>;
  hideProject: boolean;
  onEdit: () => void;
  onDelete: () => void;
  historyOpen: boolean;
  onHistory: () => void;
}) {
  const e = rec.payload;
  const s = data.state.settings.payload;
  const n = names(data, e);
  // Pieces ending at midnight show 24:00 (concept open item 35).
  const endsAtMidnight =
    e.end_at !== null && e.piece_index < e.piece_count - 1 && e.piece_count > 1;
  return (
    <>
    <tr
      tabIndex={0}
      data-record={rec.id}
      aria-label={t("ui.entries.row_label", { description: e.description })}
      onDoubleClick={onEdit}
      // Enter on the row itself edits it; on a button inside it, the button acts.
      onKeyDown={(k) => k.key === "Enter" && k.target === k.currentTarget && onEdit()}
    >
      <td>{e.start_at ? displayTime(e.start_at, e.timezone, s.clock_24h) : "–"}</td>
      <td>{e.end_at ? (endsAtMidnight ? "24:00" : displayTime(e.end_at, e.timezone, s.clock_24h)) : "–"}</td>
      <td>{formatHhmm(e.minutes)}</td>
      {!hideProject && (
        <td>
          <ColorDot color={n.color} />
          {n.project}
        </td>
      )}
      <td>{n.category}</td>
      <td>
        {e.piece_count > 1 && <span title={t("ui.entries.split_link")}>↔ </span>}
        <DescriptionText text={e.description} tags={e.tags} />
      </td>
      <td>{e.billable ? t("ui.yes") : t("ui.no")}</td>
      <td className="actions">
        <button onClick={onEdit} aria-label={t("ui.entries.edit_label", { description: e.description })}>
          {t("ui.edit")}
        </button>
        <button onClick={onHistory} aria-expanded={historyOpen}>
          {historyOpen ? t("ui.history.hide") : t("ui.history.show")}
        </button>
        <button className="danger" onClick={onDelete} aria-label={t("ui.entries.delete_label", { description: e.description })}>
          {t("ui.entries.delete")}
        </button>
      </td>
    </tr>
    {historyOpen && (
      <tr>
        <td colSpan={hideProject ? 7 : 8}>
          <HistoryPanel data={data} id={rec.id} />
        </td>
      </tr>
    )}
    </>
  );
}

function EditRow({
  data,
  rec,
  columns,
  onDone,
}: {
  data: Data;
  rec: Rec<TimeEntry>;
  columns: number;
  onDone: (changed: boolean) => Promise<void>;
}) {
  const e = rec.payload;
  const s = data.state.settings.payload;
  const interval = e.kind === "interval";
  // The whole span of a split entry is edited as one (concept §8.5).
  const firstDate = interval && e.span_start_at ? spanDate(e, data) : e.work_date;
  // Kept as typed and read when the row is saved.
  const [date, setDate] = useState(displayDate(firstDate, s.date_format));
  const [start, setStart] = useState(
    e.span_start_at ? formatClock(clockIn(e.span_start_at, e.timezone), s.clock_24h) : "",
  );
  const [end, setEnd] = useState(
    e.span_end_at ? formatClock(clockIn(e.span_end_at, e.timezone), s.clock_24h) : "",
  );
  const [duration, setDuration] = useState(formatHhmm(e.minutes));
  const [categoryId, setCategoryId] = useState(e.category_id ?? "");
  const [description, setDescription] = useState(e.description);
  const [tags, setTags] = useState(e.tags.join(", "));
  const [billable, setBillable] = useState(e.billable);
  const [error, setError] = useState<string | null>(null);

  const project = data.projects.find((p) => p.id === e.project_id);
  const categories = data.categories.filter(
    (c) => c.id === e.category_id || project?.payload.category_ids.includes(c.id),
  );

  const save = async () => {
    const iso = parseDate(date, s.date_format);
    if (iso === null) return setError(tError("invalid_date_time"));
    let timing: Timing | undefined;
    if (interval) {
      const from = parseClock(start);
      const to = parseClock(end);
      if (from === null || to === null) return setError(tError("invalid_date_time"));
      timing = { mode: "interval", date: iso, start: from, end: to };
    } else {
      const minutes = parseHhmm(duration);
      if (minutes === null) return setError(t("error.duration_out_of_range"));
      timing = { mode: "duration", date: iso, minutes };
    }
    try {
      await call("update_entry", {
        group: e.group_id,
        patch: {
          category_id: categoryId || null,
          description,
          billable,
          tags: tags.split(",").map((x) => x.trim()).filter(Boolean),
          timing,
        },
      });
      await onDone(true);
    } catch (err) {
      // The row stays open with the edited values (concept §14.3).
      setError(errorText(err));
    }
  };
  const keys = (k: React.KeyboardEvent) => {
    if (k.key === "Escape") void onDone(false);
    if (k.key === "Enter" && !k.shiftKey) {
      k.preventDefault();
      void save();
    }
  };

  return (
    <tr className="editing" onKeyDown={keys}>
      <td colSpan={columns}>
        <ErrorLine message={error} />
        <div className="row">
          <label>
            {t("ui.entries.date")}
            <DateInput autoFocus value={date} format={s.date_format} onChange={setDate} />
          </label>
          {interval ? (
            <>
              <label>
                {t("ui.entries.from")}
                <TimeInput value={start} clock24h={s.clock_24h} onChange={setStart} />
              </label>
              <label>
                {t("ui.entries.to")}
                <TimeInput value={end} clock24h={s.clock_24h} onChange={setEnd} />
              </label>
            </>
          ) : (
            <label>
              {t("ui.entries.duration")}
              <input value={duration} onChange={(x) => setDuration(x.target.value)} />
            </label>
          )}
          <label>
            {t("ui.entries.category")}
            <select value={categoryId} onChange={(x) => setCategoryId(x.target.value)}>
              <option value="">{t("ui.entries.no_category")}</option>
              {categories.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.payload.name}
                </option>
              ))}
            </select>
          </label>
          <label className="grow">
            {t("ui.entries.description")}
            <textarea
              rows={Math.min(6, Math.max(1, description.split("\n").length))}
              value={description}
              onChange={(x) => setDescription(x.target.value)}
            />
          </label>
          <label>
            {t("ui.entries.tags")}
            <input value={tags} onChange={(x) => setTags(x.target.value)} />
          </label>
          <label className="check">
            <input type="checkbox" checked={billable} onChange={(x) => setBillable(x.target.checked)} />
            {t("ui.entries.billable")}
          </label>
          <span className="actions">
            <button onClick={() => void save()}>{t("ui.save")}</button>
            <button onClick={() => void onDone(false)}>{t("ui.cancel")}</button>
          </span>
        </div>
      </td>
    </tr>
  );
}

/** Local date of the start of the whole span, in the entry's zone. */
function spanDate(e: TimeEntry, _data: Data): string {
  return new Intl.DateTimeFormat("en-CA", { timeZone: e.timezone }).format(
    new Date(e.span_start_at!),
  );
}
