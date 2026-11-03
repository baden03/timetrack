import { describe, expect, it } from "vitest";
import { composeDescription, descriptionItems, isBulletList } from "./description";

describe("descriptionItems", () => {
  it("splits a bullet list and leaves other text on one line", () => {
    expect(descriptionItems("- Setup folders\n- Add files\n- Update text")).toEqual([
      "Setup folders",
      "Add files",
      "Update text",
    ]);
    expect(descriptionItems("Fixed Safari\nlayout   bug")).toEqual(["Fixed Safari layout bug"]);
    expect(descriptionItems("- only one")).toEqual(["- only one"]);
    expect(descriptionItems("- a\nplain")).toEqual(["- a plain"]);
  });
  it("tells a list from a line", () => {
    expect(isBulletList("- a\n- b")).toBe(true);
    expect(isBulletList("a\nb")).toBe(false);
  });
});

describe("composeDescription", () => {
  it("keeps one line plain and makes several into bullets", () => {
    expect(composeDescription(["Only one"])).toBe("Only one");
    expect(composeDescription(["Setup folders", " ", "Add files"])).toBe("- Setup folders\n- Add files");
    expect(composeDescription([])).toBe("");
    expect(composeDescription(["  ", ""])).toBe("");
  });
  it("round-trips through descriptionItems", () => {
    const lines = ["One", "Two", "Three"];
    expect(descriptionItems(composeDescription(lines))).toEqual(lines);
  });
});
