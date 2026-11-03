import { descriptionItems } from "../description";

/** An entry's description: a bullet list when it is one, otherwise plain text. */
export function DescriptionText({ text }: { text: string }) {
  const items = descriptionItems(text);
  if (items.length < 2) return <>{text.includes("\n") ? items[0] : text}</>;
  return (
    <ul className="bullets">
      {items.map((item, i) => (
        <li key={`${i}-${item}`}>{item}</li>
      ))}
    </ul>
  );
}
