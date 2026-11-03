import { useCallback, useEffect, useState } from "react";
import { call, type ProjectCard, type TimerState } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "../components/Errors";
import { t } from "../i18n";
import { colorOf, themedColor, useDark } from "../colors";
import { displayDate } from "../time";

/** Project cards, most recently used first (concept §14.1). */
export function DashboardPage({
  data,
  version,
  onOpen,
  onChanged,
}: {
  data: Data;
  version: number;
  onOpen: (projectId: string) => void;
  onChanged: () => void;
}) {
  const dark = useDark();
  const [cards, setCards] = useState<ProjectCard[]>([]);
  const [timerRunning, setTimerRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const s = data.state.settings.payload;

  const load = useCallback(async () => {
    try {
      setCards(await call<ProjectCard[]>("dashboard"));
      setTimerRunning((await call<TimerState>("timer_state")).timer !== null);
      setError(null);
    } catch (e) {
      setError(errorText(e));
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load, version]);

  const start = async (projectId: string) => {
    try {
      await call("start_timer", { timer: { project_id: projectId } });
      onChanged();
    } catch (e) {
      setError(errorText(e));
    }
  };

  return (
    <div>
      <h1>{t("ui.dashboard.title")}</h1>
      <ErrorLine message={error} />
      {cards.length === 0 ? (
        <p className="muted">{t("ui.dashboard.empty")}</p>
      ) : (
        <ul className="cards">
          {cards.map((card) => (
            <li key={card.project.id}>
              <article
                className={`project-card${card.running ? " running" : ""}${colorOf(card.project.payload, { color: card.client_color }) ? " colored" : ""}`}
                style={(() => {
                  const c = colorOf(card.project.payload, { color: card.client_color });
                  return c ? ({ "--card-color": themedColor(c, dark) } as React.CSSProperties) : undefined;
                })()}
              >
                <button className="card-link" onClick={() => onOpen(card.project.id)}>
                  <span className="client">{card.client_name}</span>
                  <span className="project">{card.project.payload.name}</span>
                </button>
                <span className="muted last">
                  {card.running
                    ? t("ui.dashboard.running")
                    : t("ui.dashboard.last", {
                        date: displayDate(
                          new Intl.DateTimeFormat("en-CA", { timeZone: s.timezone }).format(
                            new Date(card.last_activity),
                          ),
                          s.date_format,
                        ),
                      })}
                </span>
                <button
                  className="start"
                  disabled={timerRunning}
                  title={timerRunning ? t("ui.dashboard.timer_busy") : undefined}
                  onClick={() => void start(card.project.id)}
                >
                  {t("ui.timer.start")}
                </button>
              </article>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
