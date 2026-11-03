import { useEffect, useState } from "react";
import { call, type HistoryEntry, type Id } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "./Errors";
import { t } from "../i18n";

/** Before and after values of every change to one stored record. */
export function HistoryPanel({ data, id }: { data: Data; id: Id }) {
  const [history, setHistory] = useState<HistoryEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    call<HistoryEntry[]>("history", { id })
      .then(setHistory)
      .catch((e) => setError(errorText(e)));
  }, [id]);

  const settings = data.state.settings.payload;
  const stamp = (instant: string) =>
    new Intl.DateTimeFormat(settings.clock_24h ? "en-GB" : "en-US", {
      timeZone: settings.timezone,
      dateStyle: "short",
      timeStyle: "medium",
    }).format(new Date(instant));
  const label = (field: string) => {
    const key = `ui.field.${field}`;
    const text = t(key);
    return text === key ? field : text;
  };
  const show = (field: string, value: unknown): string => {
    if (value === null || value === undefined || value === "") return t("ui.history.empty");
    if (field === "project_id")
      return data.projects.find((p) => p.id === value)?.payload.name ?? String(value);
    if (field === "category_id")
      return data.categories.find((c) => c.id === value)?.payload.name ?? String(value);
    if (Array.isArray(value)) return value.join(", ") || t("ui.history.empty");
    if (typeof value === "boolean") return value ? t("ui.yes") : t("ui.no");
    return String(value);
  };

  return (
    <div className="history" aria-label={t("ui.history.title")}>
      <ErrorLine message={error} />
      <ol>
        {(history ?? []).map((h) => {
          const before = h.before ?? {};
          const after = h.after ?? {};
          const fields = [...new Set([...Object.keys(before), ...Object.keys(after)])].filter(
            (f) => h.op === "revise" && JSON.stringify(before[f]) !== JSON.stringify(after[f]),
          );
          return (
            <li key={h.revision}>
              <strong>{t(`ui.history.${h.op}`)}</strong>{" "}
              <span className="muted">{stamp(h.at)}</span>
              <ul>
                {fields.map((f) => (
                  <li key={f}>
                    {t("ui.history.change", {
                      field: label(f),
                      before: show(f, before[f]),
                      after: show(f, after[f]),
                    })}
                  </li>
                ))}
              </ul>
            </li>
          );
        })}
      </ol>
    </div>
  );
}
