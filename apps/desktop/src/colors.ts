import { createContext, useContext, useEffect, useState } from "react";

/** Colors offered for clients and projects: name key and the stored `#rrggbb`. */
export const PALETTE: { key: string; hex: string }[] = [
  { key: "red", hex: "#e5484d" },
  { key: "orange", hex: "#f76b15" },
  { key: "amber", hex: "#e5a100" },
  { key: "green", hex: "#30a46c" },
  { key: "teal", hex: "#12a594" },
  { key: "blue", hex: "#3e63dd" },
  { key: "indigo", hex: "#6e56cf" },
  { key: "purple", hex: "#ab4aba" },
  { key: "pink", hex: "#d6409f" },
  { key: "brown", hex: "#a2845e" },
  { key: "gray", hex: "#8b8d98" },
];

function toHsl(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16);
  const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => v / 255);
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h = 0;
  if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
  else if (max === g) h = (b - r) / d + 2;
  else h = (r - g) / d + 4;
  return [h * 60, s, l];
}

function fromHsl(h: number, s: number, l: number): string {
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    const a = s * Math.min(l, 1 - l);
    const c = l - a * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    return Math.round(c * 255)
      .toString(16)
      .padStart(2, "0");
  };
  return `#${f(0)}${f(8)}${f(4)}`;
}

export const LIGHT_BACKGROUND = "#fafafa";
export const DARK_BACKGROUND = "#121214";
const TARGET_CONTRAST = 3.2;

/**
 * A stored color adjusted to the theme: the lightness is moved (darker on the
 * light theme, lighter on the dark one) just far enough that the color stands
 * out against the background (3:1 at least), whichever shade was picked. The
 * hue and saturation are kept, and a color that already works is unchanged.
 */
export function themedColor(hex: string, dark: boolean): string {
  const [h, s, l0] = toHsl(hex);
  const background = dark ? DARK_BACKGROUND : LIGHT_BACKGROUND;
  let l = l0;
  let candidate = fromHsl(h, s, l);
  for (let i = 0; i < 100 && contrast(candidate, background) < TARGET_CONTRAST; i++) {
    l = Math.min(1, Math.max(0, l + (dark ? 0.01 : -0.01)));
    candidate = fromHsl(h, s, l);
  }
  return candidate;
}

/** WCAG contrast ratio of two `#rrggbb` colors. */
export function contrast(a: string, b: string): number {
  const lum = (hex: string) => {
    const n = parseInt(hex.slice(1), 16);
    const [r, g, bl] = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => {
      const c = v / 255;
      return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
  };
  const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
}

/** The color of a project: its own, else its client's, else none. */
export function colorOf(
  project: { color?: string | null } | undefined,
  client: { color?: string | null } | undefined,
): string | null {
  return project?.color ?? client?.color ?? null;
}

/** True when the dark theme is showing (chosen, or the system's). */
export function useIsDark(theme: "system" | "light" | "dark"): boolean {
  const query = typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: dark)") : null;
  const [system, setSystem] = useState(query?.matches ?? false);
  useEffect(() => {
    if (!query) return;
    const listener = (e: MediaQueryListEvent) => setSystem(e.matches);
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, [query]);
  return theme === "dark" || (theme === "system" && system);
}

export const DarkContext = createContext(false);
export const useDark = () => useContext(DarkContext);
