import { useEffect, useState } from "react";
import { appVersion, COPYRIGHT_HOLDER, COPYRIGHT_YEAR, openRepository, REPOSITORY_URL } from "../about";
import { t } from "../i18n";

/** What the app is, which version this is, where the source lives. */
export function AboutPage() {
  const [version, setVersion] = useState("");
  useEffect(() => {
    void appVersion().then(setVersion);
  }, []);
  return (
    <div className="about">
      <h1>{t("ui.about.title")}</h1>
      <p className="tagline">{t("ui.about.tagline")}</p>
      <dl className="facts">
        <div>
          <dt>{t("ui.about.version_label")}</dt>
          <dd>{version}</dd>
        </div>
        <div>
          <dt>{t("ui.about.source_label")}</dt>
          <dd>
            <a
              href={REPOSITORY_URL}
              onClick={(e) => {
                e.preventDefault();
                void openRepository();
              }}
            >
              {REPOSITORY_URL.replace("https://", "")}
            </a>
          </dd>
        </div>
        <div>
          <dt>{t("ui.about.license_label")}</dt>
          <dd>{t("ui.about.license")}</dd>
        </div>
      </dl>
      <p className="muted">{t("ui.about.copyright", { year: COPYRIGHT_YEAR, holder: COPYRIGHT_HOLDER })}</p>
      <p className="muted">{t("ui.about.local_only")}</p>
    </div>
  );
}
