import { useCallback, useEffect, useState } from "react";
import {
  call,
  type AppState,
  type Category,
  type Client,
  type Project,
  type Rec,
} from "./api";
import { tError, t } from "./i18n";
import { CommandError } from "./api";
import { EntriesPage } from "./pages/EntriesPage";
import { FirstRun } from "./pages/FirstRun";
import { SetupPage } from "./pages/SetupPage";
import { SettingsPage } from "./pages/SettingsPage";

export interface Data {
  state: AppState;
  clients: Rec<Client>[];
  projects: Rec<Project>[];
  categories: Rec<Category>[];
}

type Page = "entries" | "setup" | "settings";

export function App() {
  const [data, setData] = useState<Data | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState<Page>("entries");

  const reload = useCallback(async () => {
    try {
      const [state, clients, projects, categories] = await Promise.all([
        call<AppState>("state"),
        call<Rec<Client>[]>("list", { kind: "client", include_archived: true }),
        call<Rec<Project>[]>("list", { kind: "project", include_archived: true }),
        call<Rec<Category>[]>("list", { kind: "category", include_archived: true }),
      ]);
      setData({ state, clients, projects, categories });
      setError(null);
    } catch (e) {
      setError(tError(e instanceof CommandError ? e.code : "storage"));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const theme = data?.state.settings.payload.theme ?? "system";
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  if (error) return <p role="alert" className="error">{error}</p>;
  if (!data) return <p>{t("ui.loading")}</p>;
  if (data.state.first_run) return <FirstRun onDone={reload} />;

  return (
    <div className="shell">
      <nav aria-label={t("ui.nav.label")}>
        <strong className="brand">{t("ui.app.name")}</strong>
        {(["entries", "setup", "settings"] as Page[]).map((p) => (
          <button
            key={p}
            aria-current={page === p ? "page" : undefined}
            onClick={() => setPage(p)}
          >
            {t(`ui.nav.${p}`)}
          </button>
        ))}
      </nav>
      <main>
        {page === "entries" && <EntriesPage data={data} />}
        {page === "setup" && <SetupPage data={data} reload={reload} />}
        {page === "settings" && <SettingsPage data={data} reload={reload} />}
      </main>
    </div>
  );
}
