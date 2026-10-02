import { cleanup, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { BoardGrid } from "@/features/widget-board";
import { DEFAULT_APPEARANCE, DEFAULT_SECTION_APPEARANCE, type WidgetHeight } from "@/shared/api";
import { renderWithProviders } from "@/shared/lib/testing";

import { mockGeometry, serve } from "./testing";

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/layout/",
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
  useSearchParams: () => new URLSearchParams(),
}));

const place = { appearance: DEFAULT_APPEARANCE, environments: null, public: false, settings: {}, section: "main" };

const OWNER = {
  sections: [{ id: "main", title: null, appearance: DEFAULT_SECTION_APPEARANCE }],
  widgets: [
    { ...place, key: "#0", type: "status-summary", id: "media", title: "Media", column: 1, row: 3, width: 6, height: 4 },
    { ...place, key: "#1", type: "status-summary", id: "network", title: "Network", column: 7, row: 3, width: 6, height: 4 },
    { ...place, key: "#2", type: "status-summary", id: "custom", title: "Custom", column: 1, row: 8, width: 12, height: "auto" },
    { ...place, key: "#3", type: "status-summary", id: "summary", title: "цацацац", column: 1, row: 1, width: 12, height: 2 },
  ],
};

beforeEach(mockGeometry);

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const cells = (selector: string) =>
  [...document.querySelectorAll<HTMLElement>(selector)].map((cell) => [cell.style.getPropertyValue("--span"), cell.style.getPropertyValue("--rows"), cell.style.getPropertyValue("--column")]);

it("the editor draws the owner's layout in the home page's order, with the same cells", async () => {
  serve({ layout: OWNER });
  await waitFor(() => expect(document.querySelectorAll("[data-widget]")).toHaveLength(4));
  const editorOrder = [...document.querySelectorAll("[data-widget]")].map((tile) => tile.getAttribute("data-widget"));
  expect(editorOrder).toEqual(["#3", "#0", "#1", "#2"]);
  const editorCells = cells("[data-widget]");
  cleanup();
  vi.restoreAllMocks();
  const sameContent = 2 * 96 - 16;
  vi.spyOn(Element.prototype, "getBoundingClientRect").mockReturnValue({ x: 0, y: 0, left: 0, top: 0, width: 600, height: sameContent, right: 600, bottom: sameContent, toJSON: () => ({}) } as DOMRect);
  const widgets = OWNER.widgets.map(({ key, type, id, title, settings, section, width, height, column, row, appearance }) => ({ key, type, id, title, settings, section, width, height: height as WidgetHeight, column, row, appearance }));
  renderWithProviders(<BoardGrid sections={OWNER.sections} widgets={widgets} services={[]} />);
  const boardCells = cells("[data-section='main'] > div > [data-width]");
  expect(boardCells).toEqual(editorCells);
  expect(boardCells[0]).toEqual(["12", "24", "1"]);
});
