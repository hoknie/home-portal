import { render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { rowsFor } from "./grid";
import { rememberedRows, useAutoRows } from "./use-auto-rows";

const committed: number[] = [];

function Probe({ memory }: { memory?: string }) {
  const [ref, rows] = useAutoRows<HTMLDivElement>(true, memory);
  committed.push(rows);
  return <div ref={ref} data-rows={rows} />;
}

afterEach(() => {
  vi.restoreAllMocks();
  committed.length = 0;
});

function contentOf(height: number) {
  vi.spyOn(Element.prototype, "getBoundingClientRect").mockReturnValue({ x: 0, y: 0, left: 0, top: 0, width: 300, height, right: 300, bottom: height, toJSON: () => ({}) } as DOMRect);
}

it("an automatic height is measured before the first paint, so the widget never shows one step tall", () => {
  contentOf(100);
  const { container } = render(<Probe />);
  expect(container.querySelector("[data-rows]")).toHaveAttribute("data-rows", String(rowsFor(100)));
});

it("a widget that mounts again starts at its remembered height", () => {
  contentOf(240);
  const first = render(<Probe memory="#7" />);
  expect(rememberedRows("#7")).toBe(rowsFor(240));
  first.unmount();
  committed.length = 0;
  render(<Probe memory="#7" />);
  expect(committed[0]).toBe(rowsFor(240));
});
