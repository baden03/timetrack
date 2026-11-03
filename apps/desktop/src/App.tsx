import { useCallback, useEffect, useState } from "react";
import {
  call,
  inTauri,
  type AppState,
  type Category,
  type Client,
  type Project,
  type Rec,
} from "./api";
import { DarkContext, useIsDark } from "./colors";
import { setLanguage, tError, t } from "./i18n";
import { CommandError } from "./api";
import { EntriesPage } from "./pages/EntriesPage";
import { FirstRun } from "./pages/FirstRun";
import { DashboardPage } from "./pages/DashboardPage";
import { ProjectPage } from "./pages/ProjectPage";
import { AboutPage } from "./pages/AboutPage";
import { SearchPage } from "./pages/SearchPage";
import { BackupReminder } from "./components/BackupReminder";
import { TimerBar } from "./components/TimerBar";
import { SummaryPage } from "./pages/SummaryPage";
import { SetupPage } from "./pages/SetupPage";
import { SettingsPage } from "./pages/SettingsPage";

export interface Data {
  state: AppState;
  clients: Rec<Client>[];
  projects: Rec<Project>[];
  categories: Rec<Category>[];
}

type Page =
  | "dashboard"
  | "project"
  | "entries"
  | "search"
  | "summary"
  | "setup"
  | "settings"
  | "about";

export function App() {
  const [data, setData] = useState<Data | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState<Page>("dashboard");
  const [projectId, setProjectId] = useState<string | null>(null);
  // Bumped when something outside the entry list (the timer) changes entries.
  const [version, setVersion] = useState(0);

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

  // One path for every change to the timer, wherever it came from (this
  // window, the shortcut, the menu bar): reload everything that shows it.
  // Focusing the window reloads too, in case it missed an event while hidden.
  useEffect(() => {
    const bump = () => setVersion((v) => v + 1);
    window.addEventListener("focus", bump);
    const unlisten: Array<() => void> = [];
    if (inTauri) {
      void import("@tauri-apps/api/event").then(async ({ listen }) => {
        unlisten.push(await listen("timer-changed", bump));
      });
    }
    return () => {
      window.removeEventListener("focus", bump);
      unlisten.forEach((u) => u());
    };
  }, []);

  const theme = data?.state.settings.payload.theme ?? "system";
  const dark = useIsDark(theme);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  if (error) return <p role="alert" className="error">{error}</p>;
  if (!data) return <p>{t("ui.loading")}</p>;
  // Every message below is looked up in the language chosen in Settings.
  setLanguage(data.state.settings.payload.language);
  if (data.state.first_run) return <FirstRun onDone={reload} />;

  return (
    <DarkContext.Provider value={dark}>
    <div className="shell">
      <a className="skip" href="#main">
        {t("ui.skip")}
      </a>
      <nav aria-label={t("ui.nav.label")}>
        <strong className="brand">{t("ui.app.name")}</strong>
        {(["dashboard", "entries", "search", "summary", "setup", "settings", "about"] as Page[]).map((p) => (
          <button
            key={p}
            aria-current={page === p || (p === "dashboard" && page === "project") ? "page" : undefined}
            onClick={() => setPage(p)}
          >
            {t(`ui.nav.${p}`)}
          </button>
        ))}
      </nav>
      <main id="main" tabIndex={-1}>
        <BackupReminder data={data} reload={reload} />
        <TimerBar
          data={data}
          version={version}
          showIdle={page !== "dashboard" && page !== "project"}
          onChanged={() => setVersion((v) => v + 1)} />
        {page === "dashboard" && (
          <DashboardPage
            data={data}
            version={version}
            onOpen={(id) => {
              setProjectId(id);
              setPage("project");
            }}
            onChanged={() => setVersion((v) => v + 1)}
          />
        )}
        {page === "project" && projectId && (
          <ProjectPage
            data={data}
            projectId={projectId}
            version={version}
            onBack={() => setPage("dashboard")}
            onChanged={() => setVersion((v) => v + 1)}
          />
        )}
        {page === "entries" && <EntriesPage data={data} version={version} />}
        {page === "search" && <SearchPage data={data} />}
        {page === "summary" && <SummaryPage data={data} />}
        {page === "about" && <AboutPage />}
        {page === "setup" && <SetupPage data={data} reload={reload} />}
        {page === "settings" && <SettingsPage data={data} reload={reload} />}
      </main>
    </div>
    </DarkContext.Provider>
  );
}
