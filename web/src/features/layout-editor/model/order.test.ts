import { expect, it } from "vitest";

import { DEFAULT_SECTION_APPEARANCE } from "@/shared/api";

import type { Draft, DraftWidget } from "./draft";
import { ordered, placedAt, standingOf, steppedPlace } from "./order";

const widget = (uid: string, section: string, column: number | null, row: number | null, width = 6): DraftWidget => ({ uid, key: uid, widget: uid, section, column, row, width, height: "auto" });

const owner: Draft = {
  sections: [
    { id: "main", title: null, appearance: DEFAULT_SECTION_APPEARANCE },
    { id: "more", title: null, appearance: DEFAULT_SECTION_APPEARANCE },
  ],
  widgets: [widget("media", "main", 1, 3), widget("network", "main", 7, 3), widget("custom", "main", 1, 8, 12), widget("summary", "main", 1, 1, 12), widget("roads", "more", null, null, 12)],
};

const uids = (draft: Draft, section: string) => ordered(draft, section).map((item) => item.uid);

it("orders a section by row then column, whatever the file order", () => {
  expect(uids(owner, "main")).toEqual(["summary", "media", "network", "custom"]);
});

it("places a widget before another and renumbers the section in the order shown", () => {
  const moved = placedAt(owner, "custom", { section: "main", column: 1, before: "media" });
  expect(uids(moved, "main")).toEqual(["summary", "custom", "media", "network"]);
  expect(ordered(moved, "main").map((item) => item.row)).toEqual([1, 2, 3, 4]);
});

it("places a widget at the end, keeps the column it was given and clamps it to fit its width", () => {
  const moved = placedAt(owner, "media", { section: "main", column: 10, before: null });
  expect(uids(moved, "main")).toEqual(["summary", "network", "custom", "media"]);
  expect(moved.widgets.find((item) => item.uid === "media")).toMatchObject({ column: 7, row: 4 });
});

it("moves a widget to another section, and a flowing neighbour gets its measured column", () => {
  const moved = placedAt(owner, "network", { section: "more", column: 7, before: "roads" }, { roads: 1 });
  expect(uids(moved, "main")).toEqual(["summary", "media", "custom"]);
  expect(uids(moved, "more")).toEqual(["network", "roads"]);
  expect(moved.widgets.find((item) => item.uid === "roads")).toMatchObject({ column: 1, row: 2 });
});

it("steps through the order and sideways by keyboard", () => {
  expect(standingOf(owner, "media")).toEqual({ section: "main", column: 1, index: 1, count: 4 });
  expect(steppedPlace(owner, "media", { columns: 0, rows: 1, sections: 0 })).toEqual({ section: "main", column: 1, before: "custom" });
  expect(steppedPlace(owner, "media", { columns: 0, rows: -1, sections: 0 })).toEqual({ section: "main", column: 1, before: "summary" });
  expect(steppedPlace(owner, "media", { columns: 2, rows: 0, sections: 0 })).toEqual({ section: "main", column: 3, before: "network" });
  expect(steppedPlace(owner, "custom", { columns: 0, rows: 1, sections: 0 })).toEqual({ section: "main", column: 1, before: null });
  expect(steppedPlace(owner, "media", { columns: 0, rows: 0, sections: 1 })).toEqual({ section: "more", column: 1, before: null });
  expect(steppedPlace(owner, "roads", { columns: 0, rows: 0, sections: 1 })).toBeNull();
});

it("a move onto itself changes nothing", () => {
  expect(placedAt(owner, "media", { section: "main", column: 1, before: "media" })).toBe(owner);
});
