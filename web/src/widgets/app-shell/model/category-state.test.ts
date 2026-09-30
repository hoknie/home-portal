import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { CATEGORIES_KEY, readCollapsedCategories, rereadCollapsedCategories, toggleCategory, useCollapsedCategories } from "./category-state";

beforeEach(() => {
  window.localStorage.clear();
  rereadCollapsedCategories();
});

afterEach(() => {
  vi.restoreAllMocks();
});

it("a collapsed category is stored and seen by every reader", () => {
  const sidebar = renderHook(() => useCollapsedCategories());
  const sheet = renderHook(() => useCollapsedCategories());
  expect(sidebar.result.current.size).toBe(0);
  act(() => toggleCategory("automation"));
  expect(sidebar.result.current.has("automation")).toBe(true);
  expect(sheet.result.current.has("automation")).toBe(true);
  expect(window.localStorage.getItem(CATEGORIES_KEY)).toBe('["automation"]');
  act(() => toggleCategory("automation"));
  expect(sidebar.result.current.size).toBe(0);
});

it("a stored set is read back, and broken or unknown values give every category expanded", () => {
  window.localStorage.setItem(CATEGORIES_KEY, '["network","nonsense"]');
  expect([...readCollapsedCategories()]).toEqual(["network"]);
  window.localStorage.setItem(CATEGORIES_KEY, "{broken");
  expect(readCollapsedCategories().size).toBe(0);
  window.localStorage.setItem(CATEGORIES_KEY, '{"network":true}');
  expect(readCollapsedCategories().size).toBe(0);
});

it("storage that throws leaves every category expanded and toggling still works", () => {
  vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  const view = renderHook(() => useCollapsedCategories());
  expect(view.result.current.size).toBe(0);
  act(() => toggleCategory("network"));
  expect(view.result.current.has("network")).toBe(true);
});
