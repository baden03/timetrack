import type { Settings } from "./api";

export function formatHhmm(minutes: number): string {
  const m = Math.abs(minutes);
  return `${minutes < 0 ? "-" : ""}${Math.floor(m / 60)}:${String(m % 60).padStart(2, "0")}`;
}

/** Parses `h:mm` into minutes, or null if it is not that shape. */
export function parseHhmm(text: string): number | null {
  const match = /^(\d+):([0-5]\d)$/.exec(text.trim());
  return match ? Number(match[1]) * 60 + Number(match[2]) : null;
}

/** Today's date (YYYY-MM-DD) in a zone. */
export function todayIn(zone: string, now: Date = new Date()): string {
  return new Intl.DateTimeFormat("en-CA", { timeZone: zone }).format(now);
}

export function addDays(iso: string, days: number): string {
  const d = new Date(`${iso}T12:00:00Z`);
  d.setUTCDate(d.getUTCDate() + days);
  return d.toISOString().slice(0, 10);
}

/** 24-hour `HH:MM` of an instant in a zone, for form fields. */
export function clockIn(instant: string, zone: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    timeZone: zone,
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
  }).format(new Date(instant));
}

/** Time of day as displayed, following the 24-hour preference. */
export function displayTime(instant: string, zone: string, clock24h: boolean): string {
  return new Intl.DateTimeFormat(clock24h ? "en-GB" : "en-US", {
    timeZone: zone,
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: clock24h ? "h23" : "h12",
  }).format(new Date(instant));
}

/** An ISO date in the preferred format (DD.MM.YYYY, YYYY-MM-DD or MM/DD/YYYY). */
export function displayDate(iso: string, format: Settings["date_format"]): string {
  const [y, m, d] = iso.split("-");
  if (format === "YYYY-MM-DD") return iso;
  if (format === "MM/DD/YYYY") return `${m}/${d}/${y}`;
  return `${d}.${m}.${y}`;
}

export const DATE_FORMATS = ["DD.MM.YYYY", "YYYY-MM-DD", "MM/DD/YYYY"];

/** Monday of the week containing an ISO date. */
export function weekStart(iso: string): string {
  const day = new Date(`${iso}T12:00:00Z`).getUTCDay(); // 0 = Sunday
  return addDays(iso, -((day + 6) % 7));
}

/** Elapsed time as `h:mm:ss`. */
export function formatElapsed(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor(total / 60) % 60;
  const sec = total % 60;
  return `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
}
