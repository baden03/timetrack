import en from "../../../i18n/en.json";

const messages = en as Record<string, string>;

/** Text for a catalog key, with `{name}` parameters. A gap shows the key. */
export function t(key: string, params: Record<string, string | number> = {}): string {
  const text = messages[key] ?? key;
  return text.replace(/\{(\w+)\}/g, (_, name) => String(params[name] ?? ""));
}

/** Message for a machine-readable error code from the application service. */
export function tError(code: string): string {
  return t(`error.${code}`);
}
