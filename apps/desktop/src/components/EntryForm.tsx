import { useEffect, useRef, useState } from "react";
import { call, type SavedEntry, type Suggestion, type Timing } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "./Errors";
import { t, tError } from "../i18n";
import { displayDate, formatClock, parseClock, parseDate, parseHhmm } from "../time";
import { DateInput, TimeInput } from "./FormatInputs";

export function EntryForm({
  data,
  today,
  onSaved,
  fixedProjectId,
}: {
  data: Data;
  today: string;
  onSaved: () => Promise<void>;
  /** On a project page the project is given, not chosen. */
  fixedProjectId?: string;
}) {
  const settings = data.state.settings.payload;
  const projects = data.projects.filter((p) => p.payload.status === "active");
  const clientName = (id: string) => data.clients.find((c) => c.id === id)?.payload.name ?? "";

  const [chosenProject, setProjectId] = useState(projects[0]?.id ?? "");
  const projectId = fixedProjectId ?? chosenProject;
  const [categoryId, setCategoryId] = useState("");
  const [mode, setMode] = useState<"interval" | "duration">("interval");
  // The date and times are kept as typed and read when the form is submitted.
  const [dateText, setDateText] = useState(displayDate(today, settings.date_format));
  const date = parseDate(dateText, settings.date_format);
  const [start, setStart] = useState("");
  const [end, setEnd] = useState("");
  const [duration, setDuration] = useState("");
  const [description, setDescription] = useState("");
  const [tags, setTags] = useState("");
  const [billable, setBillable] = useState<boolean | null>(null); // null: project default
  const [error, setError] = useState<string | null>(null);
  const [warning, setWarning] = useState<string | null>(null);
  const firstField = useRef<HTMLSelectElement>(null);
  const categoryField = useRef<HTMLSelectElement>(null);

  const project = projects.find((p) => p.id === projectId);
  const categories = data.categories.filter(
    (c) => c.payload.status === "active" && project?.payload.category_ids.includes(c.id),
  );

  // Smart defaults (concept §8.6): the previous entry's end and the last category.
  const suggestFor = async (pid: string, d: string | null) => {
    if (!pid || !d) return;
    const s = await call<Suggestion>("suggest", { project_id: pid, date: d });
    setStart(s.start ? formatClock(s.start, settings.clock_24h) : "");
    const enabled = data.projects.find((p) => p.id === pid)?.payload.category_ids ?? [];
    // No category is fine; the last one used on the project is proposed.
    setCategoryId(s.category_id && enabled.includes(s.category_id) ? s.category_id : "");
  };

  useEffect(() => {
    void suggestFor(projectId, date);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId, date]);

  if (projects.length === 0) return <p className="muted">{t("ui.entries.need_project")}</p>;

  const submit = async () => {
    if (date === null) return setError(tError("invalid_date_time"));
    let timing: Timing;
    if (mode === "interval") {
      const from = parseClock(start);
      const to = parseClock(end);
      if (from === null || to === null) return setError(tError("invalid_date_time"));
      timing = { mode, date, start: from, end: to };
    } else {
      const minutes = parseHhmm(duration);
      if (minutes === null) return setError(t("error.duration_out_of_range"));
      timing = { mode, date, minutes };
    }
    try {
      const saved = await call<SavedEntry>("create_entry", {
        entry: {
          project_id: projectId,
          category_id: categoryId || null,
          description,
          billable: billable ?? undefined,
          tags: tags.split(",").map((x) => x.trim()).filter(Boolean),
          timing,
        },
      });
      setError(null);
      setWarning(saved.overlaps.length > 0 ? t("ui.entries.overlap") : null);
      setDescription("");
      setTags("");
      setEnd("");
      setDuration("");
      await onSaved();
      await suggestFor(projectId, date);
      (firstField.current ?? categoryField.current)?.focus();
    } catch (e) {
      // A failed save keeps what was typed (concept §8.7).
      setError(errorText(e));
    }
  };

  return (
    <form
      className="card entry-form"
      onSubmit={(e) => {
        e.preventDefault();
        void submit();
      }}
    >
      <ErrorLine message={error} />
      {warning && <p role="status" className="warning">{warning}</p>}
      <div className="row">
        {!fixedProjectId && (
          <label>
            {t("ui.entries.project")}
            <select ref={firstField} value={projectId} onChange={(e) => setProjectId(e.target.value)}>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {clientName(p.payload.client_id)} / {p.payload.name}
                </option>
              ))}
            </select>
          </label>
        )}
        <label>
          {t("ui.entries.category")}
          <select
            ref={categoryField}
            value={categoryId}
            onChange={(e) => setCategoryId(e.target.value)}
          >
            <option value="">{t("ui.entries.no_category")}</option>
            {categories.map((c) => (
              <option key={c.id} value={c.id}>
                {c.payload.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("ui.entries.date")}
          <DateInput value={dateText} format={settings.date_format} onChange={setDateText} />
        </label>
        <fieldset className="modes">
          <legend>{t("ui.entries.mode")}</legend>
          <label className="check">
            <input type="radio" name="entry-mode" checked={mode === "interval"} onChange={() => setMode("interval")} />
            {t("ui.entries.mode_interval")}
          </label>
          <label className="check">
            <input type="radio" name="entry-mode" checked={mode === "duration"} onChange={() => setMode("duration")} />
            {t("ui.entries.mode_duration")}
          </label>
        </fieldset>
        {mode === "interval" ? (
          <>
            <label>
              {t("ui.entries.from")}
              <TimeInput value={start} clock24h={settings.clock_24h} onChange={setStart} />
            </label>
            <label>
              {t("ui.entries.to")}
              <TimeInput value={end} clock24h={settings.clock_24h} onChange={setEnd} />
            </label>
          </>
        ) : (
          <label>
            {t("ui.entries.duration")}
            <input
              placeholder="1:30"
              value={duration}
              onChange={(e) => setDuration(e.target.value)}
            />
          </label>
        )}
      </div>
      <div className="row">
        <label className="grow">
          {t("ui.entries.description")}
          <textarea
            rows={1}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                void submit();
              }
            }}
          />
        </label>
        <label>
          {t("ui.entries.tags")}
          <input value={tags} onChange={(e) => setTags(e.target.value)} />
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={billable ?? project?.payload.billable_default ?? true}
            onChange={(e) => setBillable(e.target.checked)}
          />
          {t("ui.entries.billable")}
        </label>
        <button type="submit">{t("ui.entries.add")}</button>
      </div>
    </form>
  );
}
