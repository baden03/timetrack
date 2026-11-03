import { useEffect, useRef, useState } from "react";
import { call, type SavedEntry, type Suggestion, type Timing } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "./Errors";
import { t } from "../i18n";
import { parseHhmm } from "../time";

export function EntryForm({
  data,
  today,
  onSaved,
}: {
  data: Data;
  today: string;
  onSaved: () => Promise<void>;
}) {
  const projects = data.projects.filter((p) => p.payload.status === "active");
  const clientName = (id: string) => data.clients.find((c) => c.id === id)?.payload.name ?? "";

  const [projectId, setProjectId] = useState(projects[0]?.id ?? "");
  const [categoryId, setCategoryId] = useState("");
  const [mode, setMode] = useState<"interval" | "duration">("interval");
  const [date, setDate] = useState(today);
  const [start, setStart] = useState("");
  const [end, setEnd] = useState("");
  const [duration, setDuration] = useState("");
  const [description, setDescription] = useState("");
  const [tags, setTags] = useState("");
  const [billable, setBillable] = useState<boolean | null>(null); // null: project default
  const [error, setError] = useState<string | null>(null);
  const [warning, setWarning] = useState<string | null>(null);
  const firstField = useRef<HTMLSelectElement>(null);

  const project = projects.find((p) => p.id === projectId);
  const categories = data.categories.filter(
    (c) => c.payload.status === "active" && project?.payload.category_ids.includes(c.id),
  );

  // Smart defaults (concept §8.6): the previous entry's end and the last category.
  const suggestFor = async (pid: string, d: string, force: boolean) => {
    if (!pid) return;
    const s = await call<Suggestion>("suggest", { project_id: pid, date: d });
    if (force || !start) setStart(s.start ?? "");
    const enabled = data.projects.find((p) => p.id === pid)?.payload.category_ids ?? [];
    const pick = s.category_id && enabled.includes(s.category_id) ? s.category_id : enabled[0];
    if (pick) setCategoryId(pick);
  };

  useEffect(() => {
    void suggestFor(projectId, date, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId, date]);

  if (projects.length === 0) return <p className="muted">{t("ui.entries.need_project")}</p>;

  const submit = async () => {
    let timing: Timing;
    if (mode === "interval") {
      timing = { mode, date, start, end };
    } else {
      const minutes = parseHhmm(duration);
      if (minutes === null) return setError(t("error.duration_out_of_range"));
      timing = { mode, date, minutes };
    }
    try {
      const saved = await call<SavedEntry>("create_entry", {
        entry: {
          project_id: projectId,
          category_id: categoryId,
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
      await suggestFor(projectId, date, true);
      firstField.current?.focus();
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
        <label>
          {t("ui.entries.category")}
          <select value={categoryId} onChange={(e) => setCategoryId(e.target.value)}>
            {categories.map((c) => (
              <option key={c.id} value={c.id}>
                {c.payload.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("ui.entries.date")}
          <input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
        </label>
        <fieldset className="modes">
          <legend>{t("ui.entries.mode")}</legend>
          <label className="check">
            <input type="radio" checked={mode === "interval"} onChange={() => setMode("interval")} />
            {t("ui.entries.mode_interval")}
          </label>
          <label className="check">
            <input type="radio" checked={mode === "duration"} onChange={() => setMode("duration")} />
            {t("ui.entries.mode_duration")}
          </label>
        </fieldset>
        {mode === "interval" ? (
          <>
            <label>
              {t("ui.entries.from")}
              <input type="time" value={start} onChange={(e) => setStart(e.target.value)} />
            </label>
            <label>
              {t("ui.entries.to")}
              <input type="time" value={end} onChange={(e) => setEnd(e.target.value)} />
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
