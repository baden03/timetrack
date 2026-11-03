import { useEffect, useState } from "react";
import { call } from "../api";
import type { Data } from "../App";
import { ErrorLine, errorText } from "../components/Errors";
import { t } from "../i18n";
import { todayIn } from "../time";

interface MonthlySummary {
  text: string;
}

export function SummaryPage({ data }: { data: Data }) {
  const [month, setMonth] = useState(todayIn(data.state.settings.payload.timezone).slice(0, 7));
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!month) return;
    call<MonthlySummary>("monthly_summary", { month })
      .then((s) => {
        setText(s.text);
        setError(null);
        setCopied(false);
      })
      .catch((e) => setError(errorText(e)));
  }, [month]);

  return (
    <div>
      <h1>{t("ui.summary.title")}</h1>
      <div className="row">
        <label>
          {t("ui.summary.month")}
          <input type="month" value={month} onChange={(e) => setMonth(e.target.value)} />
        </label>
        <button
          onClick={async () => {
            await navigator.clipboard.writeText(text);
            setCopied(true);
          }}
        >
          {t("ui.summary.copy")}
        </button>
        {copied && <span role="status">{t("ui.summary.copied")}</span>}
      </div>
      <ErrorLine message={error} />
      <pre className="summary">{text}</pre>
    </div>
  );
}
