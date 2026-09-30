import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, expect, it } from "vitest";

import { DEFAULT_WIDTH, PANEL_KEY } from "./panel-width";
import { rereadPanelWidth, setPanelWidth, usePanelWidth } from "./use-panel-width";

beforeEach(() => {
  window.localStorage.clear();
  rereadPanelWidth();
});

afterEach(() => {
  window.localStorage.clear();
});

it("two readers see one change, and only a kept change is stored", () => {
  const first = renderHook(() => usePanelWidth());
  const second = renderHook(() => usePanelWidth());
  expect(first.result.current).toBe(DEFAULT_WIDTH);
  act(() => setPanelWidth(500, false));
  expect(first.result.current).toBe(500);
  expect(second.result.current).toBe(500);
  expect(window.localStorage.getItem(PANEL_KEY)).toBeNull();
  act(() => setPanelWidth(520, true));
  expect(window.localStorage.getItem(PANEL_KEY)).toBe("520");
});

it("the stored width is read when the page opens", () => {
  window.localStorage.setItem(PANEL_KEY, "600");
  rereadPanelWidth();
  expect(renderHook(() => usePanelWidth()).result.current).toBe(600);
});
