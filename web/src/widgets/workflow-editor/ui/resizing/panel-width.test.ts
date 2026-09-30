import { afterEach, expect, it, vi } from "vitest";

import { DEFAULT_WIDTH, NARROWEST, PANEL_KEY, clampWidth, readPanelWidth, widestFor, writePanelWidth } from "./panel-width";

afterEach(() => {
  vi.restoreAllMocks();
  window.localStorage.clear();
});

it("a width stays between 20rem and 70% of the row", () => {
  expect(clampWidth(100, 1000)).toBe(NARROWEST);
  expect(clampWidth(900, 1000)).toBe(700);
  expect(clampWidth(500.4, 1000)).toBe(500);
  expect(widestFor(1000)).toBe(700);
});

it("in a row too narrow for 70% the panel keeps 20rem", () => {
  expect(widestFor(300)).toBe(NARROWEST);
  expect(clampWidth(600, 300)).toBe(NARROWEST);
});

it("a width is remembered, and a missing, broken or unreadable one gives 26rem", () => {
  expect(readPanelWidth()).toBe(DEFAULT_WIDTH);
  writePanelWidth(512.6);
  expect(window.localStorage.getItem(PANEL_KEY)).toBe("513");
  expect(readPanelWidth()).toBe(513);
  window.localStorage.setItem(PANEL_KEY, "wide");
  expect(readPanelWidth()).toBe(DEFAULT_WIDTH);
  vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  expect(readPanelWidth()).toBe(DEFAULT_WIDTH);
  expect(() => writePanelWidth(600)).not.toThrow();
});
