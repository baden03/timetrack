import { describe, expect, it } from "vitest";
import {
  addDays,
  displayDate,
  formatClock,
  formatElapsed,
  formatHhmm,
  parseClock,
  parseDate,
  parseHhmm,
  weekStart,
} from "./time";

describe("dates in the preferred format", () => {
  it("displays and parses each format", () => {
    for (const format of ["DD.MM.YYYY", "YYYY-MM-DD", "MM/DD/YYYY"]) {
      expect(parseDate(displayDate("2026-11-03", format), format)).toBe("2026-11-03");
    }
    expect(displayDate("2026-11-03", "DD.MM.YYYY")).toBe("03.11.2026");
    expect(displayDate("2026-11-03", "MM/DD/YYYY")).toBe("11/03/2026");
  });
  it("rejects what is not a real date or not in the format", () => {
    expect(parseDate("31.02.2026", "DD.MM.YYYY")).toBeNull();
    expect(parseDate("2026-11-03", "DD.MM.YYYY")).toBeNull();
    expect(parseDate("3.11.2026", "DD.MM.YYYY")).toBe("2026-11-03");
    expect(parseDate("", "DD.MM.YYYY")).toBeNull();
  });
});

describe("times of day", () => {
  it("parses common ways of typing", () => {
    expect(parseClock("23:00")).toBe("23:00");
    expect(parseClock("2300")).toBe("23:00");
    expect(parseClock("9")).toBe("09:00");
    expect(parseClock("9:30")).toBe("09:30");
    expect(parseClock("930")).toBe("09:30");
    expect(parseClock("21.30")).toBe("21:30");
    expect(parseClock("21:30")).toBe("21:30");
    expect(parseClock("11pm")).toBe("23:00");
    expect(parseClock("12:15 am")).toBe("00:15");
    expect(parseClock("12 pm")).toBe("12:00");
  });
  it("rejects nonsense", () => {
    expect(parseClock("25:00")).toBeNull();
    expect(parseClock("9:75")).toBeNull();
    expect(parseClock("13pm")).toBeNull();
    expect(parseClock("abc")).toBeNull();
  });
  it("formats for the 12 or 24 hour preference", () => {
    expect(formatClock("23:05", true)).toBe("23:05");
    expect(formatClock("23:05", false)).toBe("11:05 PM");
    expect(formatClock("00:30", false)).toBe("12:30 AM");
    expect(formatClock("12:00", false)).toBe("12:00 PM");
  });
});

describe("durations and weeks", () => {
  it("formats and parses hh:mm", () => {
    expect(formatHhmm(7590)).toBe("126:30");
    expect(parseHhmm("1:30")).toBe(90);
    expect(parseHhmm("1:75")).toBeNull();
  });
  it("finds the Monday of a week", () => {
    expect(weekStart("2026-11-03")).toBe("2026-11-02"); // Tuesday
    expect(weekStart("2026-11-08")).toBe("2026-11-02"); // Sunday
    expect(addDays("2026-11-30", 1)).toBe("2026-12-01");
  });
  it("formats elapsed time like the menu bar does", () => {
    // The same vectors are tested for the menu bar in src-tauri/src/main.rs.
    expect(formatElapsed(0)).toBe("0:00:00");
    expect(formatElapsed(59_999)).toBe("0:00:59");
    expect(formatElapsed(3_725_000)).toBe("1:02:05");
    expect(formatElapsed(27 * 3600 * 1000)).toBe("27:00:00");
    expect(formatElapsed(-1000)).toBe("0:00:00");
  });
});
