import type { Settings } from "./api";

export function formatHhmm(minutes: number): string {
  const m = Math.abs(minutes);
  return `${minutes < 0 ? "-" : ""}${Math.floor(m / 60)}:${String(m % 60).padStart(2, "0")}`;
}

/**
 * Reads a duration as someone would type it, into whole minutes (or null):
 * `1:30`, `:45` (45 minutes), `1:5` (1:05), `.75` and `1.5` or `1,5` (decimal
 * hours), `1h30`, `2h`, `45m`, and a bare number: up to 24 is hours, more is
 * minutes (`2` is 2:00, `45` is 0:45).
 */
export function parseDuration(text: string): number | null {
  const s = text.trim().toLowerCase().replace(",", ".");
  if (s === "") return null;
  let m: RegExpExecArray | null;
  if (/^\d+$/.test(s)) {
    const n = Number(s);
    return n <= 24 ? n * 60 : n;
  }
  if ((m = /^(\d*):(\d{1,2})$/.exec(s))) {
    const minutes = Number(m[2]);
    return minutes > 59 ? null : Number(m[1] || 0) * 60 + minutes;
  }
  if ((m = /^(\d*)\.(\d+)$/.exec(s))) {
    return Math.round(Number(`${m[1] || 0}.${m[2]}`) * 60);
  }
  if ((m = /^(?:(\d+)\s*h\s*)?(?:(\d+)\s*(?:m|min)?)?$/.exec(s)) && (m[1] || m[2])) {
    const minutes = Number(m[2] || 0);
    return minutes > 59 && m[1] ? null : Number(m[1] || 0) * 60 + minutes;
  }
  return null;
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

function validDate(y: number, m: number, d: number): string | null {
  const date = new Date(Date.UTC(y, m - 1, d));
  if (date.getUTCFullYear() !== y || date.getUTCMonth() !== m - 1 || date.getUTCDate() !== d) return null;
  return date.toISOString().slice(0, 10);
}

/** Parses a typed date in the preferred format into ISO, or null if it is not a real date. */
export function parseDate(text: string, format: string): string | null {
  const s = text.trim();
  let m: RegExpExecArray | null;
  if (format === "YYYY-MM-DD") {
    m = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(s);
    return m ? validDate(+m[1], +m[2], +m[3]) : null;
  }
  if (format === "MM/DD/YYYY") {
    m = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(s);
    return m ? validDate(+m[3], +m[1], +m[2]) : null;
  }
  m = /^(\d{1,2})\.(\d{1,2})\.(\d{4})$/.exec(s);
  return m ? validDate(+m[3], +m[2], +m[1]) : null;
}

/**
 * Parses a typed time of day into `HH:MM` (24-hour). Accepts `23:00`, `2300`,
 * `9`, `930`, `9:30`, `9.30` and, with an am/pm suffix, `9:30 pm` or `11pm`.
 */
export function parseClock(text: string): string | null {
  const m = /^(\d{1,2})(?:[:.]?(\d{2}))?\s*([ap]m?)?$/i.exec(text.trim());
  if (!m) return null;
  let hour = +m[1];
  const minute = m[2] === undefined ? 0 : +m[2];
  const suffix = m[3]?.toLowerCase();
  if (minute > 59) return null;
  if (suffix) {
    if (hour < 1 || hour > 12) return null;
    hour = (hour % 12) + (suffix.startsWith("p") ? 12 : 0);
  }
  if (hour > 23) return null;
  return `${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")}`;
}

/** `HH:MM` as displayed, following the 24-hour preference. */
export function formatClock(hhmm: string, clock24h: boolean): string {
  if (!hhmm || clock24h) return hhmm;
  const [h, m] = hhmm.split(":").map(Number);
  return `${h % 12 === 0 ? 12 : h % 12}:${String(m).padStart(2, "0")} ${h < 12 ? "AM" : "PM"}`;
}
