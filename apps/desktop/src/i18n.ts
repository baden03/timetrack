import de from "../../../i18n/de.json";
import en from "../../../i18n/en.json";

type Catalog = Record<string, string>;

/**
 * Interface languages, each named in its own language. To add a language, add
 * a line here (see docs/translating.md).
 */
export const LANGUAGES: { code: string; name: string; messages: Catalog }[] = [
  { code: "en", name: "English", messages: en },
  { code: "de", name: "Deutsch", messages: de },
];

const catalogs: Record<string, Catalog> = Object.fromEntries(
  LANGUAGES.map((l) => [l.code, l.messages]),
);

let current = "en";

export function setLanguage(code: string): void {
  current = code in catalogs ? code : "en";
  if (typeof document !== "undefined") document.documentElement.lang = current;
}

function lookup(key: string): string | undefined {
  // A message missing in the chosen language falls back to English.
  return catalogs[current][key] || catalogs.en[key];
}

/**
 * Text for a catalog key, with `{name}` parameters. With a `count` parameter
 * the plural form is used: `key.one` for 1, otherwise `key.other`.
 */
export function t(key: string, params: Record<string, string | number> = {}): string {
  let text: string | undefined;
  if (typeof params.count === "number") {
    text = lookup(`${key}.${params.count === 1 ? "one" : "other"}`);
  }
  text ??= lookup(key) ?? key;
  return text.replace(/\{(\w+)\}/g, (_, name) => String(params[name] ?? ""));
}

/** Message for a machine-readable error code from the application service. */
export function tError(code: string): string {
  return t(`error.${code}`);
}
