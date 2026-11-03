/**
 * The separate items of an entry's description: the lines of a bullet list
 * (every line starts with "- ", two lines at least), or otherwise the whole
 * text as one line. The same rule is used for the summary text (core/model.rs).
 */
export function descriptionItems(description: string): string[] {
  const lines = description
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l !== "");
  if (lines.length >= 2 && lines.every((l) => l.startsWith("- "))) {
    return lines.map((l) => l.slice(2).trim());
  }
  return [description.split(/\s+/).filter(Boolean).join(" ")];
}

export function isBulletList(description: string): boolean {
  return descriptionItems(description).length > 1;
}

/**
 * Builds an entry's description from the lines typed one after another: one
 * line stays as it is, several become bullet points. The same rule is used by
 * the timer's notes (core/model.rs, `compose_description`).
 */
export function composeDescription(lines: string[]): string {
  const items = lines.map((l) => l.trim()).filter((l) => l !== "");
  return items.length < 2 ? (items[0] ?? "") : items.map((l) => `- ${l}`).join("\n");
}
