import { useState } from "react";
import { call, type SearchQuery, type SearchResult } from "../api";
import type { Data } from "../App";
import { DescriptionText } from "../components/DescriptionText";
import { ErrorLine, errorText } from "../components/Errors";
import { t, tError } from "../i18n";
import { colorOf } from "../colors";
import { ColorDot } from "../components/Color";
import { DateInput } from "../components/FormatInputs";
import { displayDate, displayTime, formatHhmm, parseDate } from "../time";

export function SearchPage({ data }: { data: Data }) {
  const s = data.state.settings.payload;
  const [q, setQ] = useState<SearchQuery>({});
  // Dates are kept as typed and read when the search runs.
  const [fromText, setFromText] = useState("");
  const [toText, setToText] = useState("");
  const [billable, setBillable] = useState("");
  const [result, setResult] = useState<SearchResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const set = (patch: SearchQuery) => setQ({ ...q, ...patch });
  const clean = (v: string) => (v === "" ? undefined : v);

  const run = async () => {
    try {
      const from = fromText.trim() === "" ? undefined : parseDate(fromText, s.date_format);
      const to = toText.trim() === "" ? undefined : parseDate(toText, s.date_format);
      if (from === null || to === null) return setError(tError("invalid_date_time"));
      const query: SearchQuery = {
        ...q,
        from,
        to,
        billable: billable === "" ? undefined : billable === "yes",
      };
      setResult(await call<SearchResult>("search_entries", { query }));
      setError(null);
    } catch (e) {
      setError(errorText(e));
    }
  };
  const name = (list: { id: string; payload: { name: string } }[], id: string) =>
    list.find((x) => x.id === id)?.payload.name ?? "";

  return (
    <div>
      <h1>{t("ui.search.title")}</h1>
      <form
        className="card"
        onSubmit={(e) => {
          e.preventDefault();
          void run();
        }}
      >
        <div className="row">
          <label className="grow">
            {t("ui.search.text")}
            <input autoFocus value={q.text ?? ""} onChange={(e) => set({ text: clean(e.target.value) })} />
          </label>
          <label>
            {t("ui.search.client")}
            <select value={q.client_id ?? ""} onChange={(e) => set({ client_id: clean(e.target.value) })}>
              <option value="">{t("ui.search.any")}</option>
              {data.clients.map((c) => (
                <option key={c.id} value={c.id}>{c.payload.name}</option>
              ))}
            </select>
          </label>
          <label>
            {t("ui.search.project")}
            <select value={q.project_id ?? ""} onChange={(e) => set({ project_id: clean(e.target.value) })}>
              <option value="">{t("ui.search.any")}</option>
              {data.projects.map((p) => (
                <option key={p.id} value={p.id}>{p.payload.name}</option>
              ))}
            </select>
          </label>
          <label>
            {t("ui.search.category")}
            <select value={q.category_id ?? ""} onChange={(e) => set({ category_id: clean(e.target.value) })}>
              <option value="">{t("ui.search.any")}</option>
              {data.categories.map((c) => (
                <option key={c.id} value={c.id}>{c.payload.name}</option>
              ))}
            </select>
          </label>
        </div>
        <div className="row">
          <label>
            {t("ui.search.tag")}
            <input value={q.tag ?? ""} onChange={(e) => set({ tag: clean(e.target.value) })} />
          </label>
          <label>
            {t("ui.search.billable")}
            <select value={billable} onChange={(e) => setBillable(e.target.value)}>
              <option value="">{t("ui.search.any")}</option>
              <option value="yes">{t("ui.yes")}</option>
              <option value="no">{t("ui.no")}</option>
            </select>
          </label>
          <label>
            {t("ui.search.from")}
            <DateInput value={fromText} format={s.date_format} onChange={setFromText} />
          </label>
          <label>
            {t("ui.search.to")}
            <DateInput value={toText} format={s.date_format} onChange={setToText} />
          </label>
          <button type="submit">{t("ui.search.run")}</button>
        </div>
      </form>
      <ErrorLine message={error} />
      {result && (
        <div>
          <p>
            <strong>
              {t("ui.search.total", { count: result.pieces.length, total: formatHhmm(result.total_minutes) })}
            </strong>
          </p>
          {result.pieces.length === 0 ? (
            <p className="muted">{t("ui.search.none")}</p>
          ) : (
            <table>
              <thead>
                <tr>
                  <th>{t("ui.entries.date")}</th>
                  <th>{t("ui.entries.from")}</th>
                  <th>{t("ui.entries.duration")}</th>
                  <th>{t("ui.entries.project")}</th>
                  <th>{t("ui.entries.category")}</th>
                  <th>{t("ui.entries.description")}</th>
                </tr>
              </thead>
              <tbody>
                {result.pieces.map((r) => {
                  const e = r.payload;
                  const project = data.projects.find((p) => p.id === e.project_id);
                  return (
                    <tr key={r.id}>
                      <td>{displayDate(e.work_date, s.date_format)}</td>
                      <td>{e.start_at ? displayTime(e.start_at, e.timezone, s.clock_24h) : "–"}</td>
                      <td>{formatHhmm(e.minutes)}</td>
                      <td>
                        <ColorDot
                          color={colorOf(
                            project?.payload,
                            data.clients.find((c) => c.id === project?.payload.client_id)?.payload,
                          )}
                        />
                        {name(data.clients, project?.payload.client_id ?? "")} / {project?.payload.name}
                      </td>
                      <td>{name(data.categories, e.category_id ?? "")}</td>
                      <td>
                        <DescriptionText text={e.description} tags={e.tags} />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  );
}
