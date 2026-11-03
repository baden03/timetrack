import { descriptionItems } from "../description";
import { t } from "../i18n";

/**
 * An entry's description (a bullet list when it is one, otherwise plain text),
 * with its tags, if any, appended after it.
 */
export function DescriptionText({ text, tags = [] }: { text: string; tags?: string[] }) {
  const items = descriptionItems(text);
  const body =
    items.length < 2 ? (
      <>{text.includes("\n") ? items[0] : text}</>
    ) : (
      <ul className="bullets">
        {items.map((item, i) => (
          <li key={`${i}-${item}`}>{item}</li>
        ))}
      </ul>
    );
  return (
    <>
      {body}
      {tags.length > 0 && (
        <span className="tags" aria-label={t("ui.entries.tags_label")}>
          {tags.map((tag) => (
            <span key={tag} className="tag">
              {tag}
            </span>
          ))}
        </span>
      )}
    </>
  );
}
