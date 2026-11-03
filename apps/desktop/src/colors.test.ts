import { describe, expect, it } from "vitest";
import { colorOf, contrast, DARK_BACKGROUND, LIGHT_BACKGROUND, PALETTE, themedColor } from "./colors";

describe("themed colors", () => {
  it("every palette color stands out against both backgrounds (3:1)", () => {
    for (const { key, hex } of PALETTE) {
      expect(contrast(themedColor(hex, false), LIGHT_BACKGROUND), `${key} light`).toBeGreaterThanOrEqual(3);
      expect(contrast(themedColor(hex, true), DARK_BACKGROUND), `${key} dark`).toBeGreaterThanOrEqual(3);
    }
  });

  it("any picked shade is brought into range, and the hue is kept", () => {
    for (const hex of ["#000000", "#ffffff", "#ffff00", "#0000ff", "#7f7f7f"]) {
      expect(contrast(themedColor(hex, false), LIGHT_BACKGROUND), hex).toBeGreaterThanOrEqual(3);
      expect(contrast(themedColor(hex, true), DARK_BACKGROUND), hex).toBeGreaterThanOrEqual(3);
    }
    // A color that already works is left alone.
    expect(themedColor("#3e63dd", true)).toBe(themedColor(themedColor("#3e63dd", true), true));
    // A red stays reddish.
    const red = themedColor("#e5484d", false);
    expect(parseInt(red.slice(1, 3), 16)).toBeGreaterThan(parseInt(red.slice(3, 5), 16));
  });

  it("a project's own color wins over its client's", () => {
    expect(colorOf({ color: "#111111" }, { color: "#222222" })).toBe("#111111");
    expect(colorOf({ color: null }, { color: "#222222" })).toBe("#222222");
    expect(colorOf({}, {})).toBeNull();
  });
});
