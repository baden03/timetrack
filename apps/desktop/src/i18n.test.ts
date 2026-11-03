import { beforeEach, describe, expect, it } from "vitest";
import { LANGUAGES, setLanguage, t, tError } from "./i18n";

const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

const english = LANGUAGES.find((l) => l.code === "en")!.messages;

describe("catalogs", () => {
  it.each(LANGUAGES.filter((l) => l.code !== "en").map((l) => [l.code, l.messages] as const))(
    "%s has exactly the keys English has, with the same placeholders",
    (_code, messages) => {
      expect(Object.keys(messages)).toEqual(Object.keys(english));
      for (const [key, text] of Object.entries(english)) {
        const other = messages[key];
        expect(other.trim(), key).not.toBe("");
        expect(placeholders(other), key).toEqual(placeholders(text));
      }
    },
  );

  it("language codes are unique and English is the base", () => {
    const codes = LANGUAGES.map((l) => l.code);
    expect(new Set(codes).size).toBe(codes.length);
    expect(codes[0]).toBe("en");
  });
});

describe("t()", () => {
  beforeEach(() => setLanguage("en"));

  it("fills parameters", () => {
    expect(t("ui.dashboard.last", { date: "03.11.2026" })).toBe("Last used 03.11.2026");
  });

  it("chooses the plural form from count", () => {
    expect(t("ui.search.total", { count: 1, total: "1:00" })).toBe("1 entry, 1:00");
    expect(t("ui.search.total", { count: 3, total: "4:30" })).toBe("3 entries, 4:30");
    expect(t("ui.search.total", { count: 0, total: "0:00" })).toBe("0 entries, 0:00");
  });

  it("switches language and uses German plurals", () => {
    setLanguage("de");
    expect(t("ui.entries.add")).toBe("Eintrag hinzufügen");
    expect(t("ui.search.total", { count: 1, total: "1:00" })).toBe("1 Eintrag, 1:00");
    expect(t("ui.search.total", { count: 2, total: "2:00" })).toBe("2 Einträge, 2:00");
    expect(tError("name_required")).toBe("Bitte gib einen Namen ein.");
  });

  it("falls back to English for an unknown language and never shows a raw key", () => {
    setLanguage("xx");
    expect(t("ui.entries.add")).toBe("Add entry");
    // Only a key that exists in no catalog is shown as itself.
    const missing = ["not", "a", "real", "key"].join(".");
    expect(t(missing)).toBe(missing);
  });
});
