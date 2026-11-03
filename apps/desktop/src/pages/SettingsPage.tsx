import { useState } from "react";
import { call, type Settings } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "../components/Errors";
import { t } from "../i18n";
import { DATE_FORMATS } from "../time";

export function SettingsPage({ data, reload }: { data: Data; reload: () => Promise<void> }) {
  const [s, setS] = useState<Settings>(data.state.settings.payload);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const zones = (Intl as unknown as { supportedValuesOf(k: string): string[] }).supportedValuesOf(
    "timeZone",
  );
  const set = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setS({ ...s, [key]: value });
    setSaved(false);
  };
  return (
    <form
      className="card"
      onSubmit={async (e) => {
        e.preventDefault();
        try {
          await call("update_settings", { settings: s });
          setError(null);
          setSaved(true);
          await reload();
        } catch (err) {
          setError(errorText(err));
        }
      }}
    >
      <h1>{t("ui.settings.title")}</h1>
      <ErrorLine message={error} />
      <label>
        {t("ui.settings.timezone")}
        <select value={s.timezone} onChange={(e) => set("timezone", e.target.value)}>
          {[...new Set([s.timezone, ...zones])].map((z) => (
            <option key={z}>{z}</option>
          ))}
        </select>
      </label>
      <label>
        {t("ui.settings.date_format")}
        <select value={s.date_format} onChange={(e) => set("date_format", e.target.value)}>
          {DATE_FORMATS.map((f) => (
            <option key={f}>{f}</option>
          ))}
        </select>
      </label>
      <label className="check">
        <input
          type="checkbox"
          checked={s.clock_24h}
          onChange={(e) => set("clock_24h", e.target.checked)}
        />
        {t("ui.settings.clock_24h")}
      </label>
      <label>
        {t("ui.settings.theme")}
        <select value={s.theme} onChange={(e) => set("theme", e.target.value as Settings["theme"])}>
          <option value="system">{t("ui.settings.theme_system")}</option>
          <option value="light">{t("ui.settings.theme_light")}</option>
          <option value="dark">{t("ui.settings.theme_dark")}</option>
        </select>
      </label>
      <label>
        {t("ui.settings.window")}
        <input
          type="number"
          min={1}
          value={s.entry_window_days}
          onChange={(e) => set("entry_window_days", Math.max(1, Number(e.target.value) || 60))}
        />
      </label>
      <label>
        {t("ui.settings.language")}
        <select value={s.language} disabled>
          <option value="en">English</option>
        </select>
      </label>
      <div className="actions">
        <button type="submit">{t("ui.save")}</button>
        {saved && <span role="status">{t("ui.saved")}</span>}
      </div>
      <p className="muted">{t("ui.settings.identity", { origin: data.state.identity.origin })}</p>
    </form>
  );
}
