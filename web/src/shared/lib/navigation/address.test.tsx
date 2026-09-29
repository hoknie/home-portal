import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { guardLeaving, pushAddress, replaceAddress, useAddress } from "./address";

beforeEach(() => {
  window.history.replaceState(null, "", "/admin/workflows/");
});

afterEach(() => {
  vi.restoreAllMocks();
});

it("reads the path after mount and follows pushes and replacements", () => {
  const { result } = renderHook(() => useAddress());
  expect(result.current.path).toBe("/admin/workflows/");
  act(() => {
    pushAddress("/admin/workflows/revive/");
  });
  expect(result.current.path).toBe("/admin/workflows/revive/");
  act(() => {
    replaceAddress("/admin/workflows/revive/history/");
  });
  expect(result.current.path).toBe("/admin/workflows/revive/history/");
  expect(window.history.length).toBeGreaterThan(1);
});

it("a refusing guard keeps the address, and a released one lets it go", () => {
  const { result } = renderHook(() => useAddress());
  const release = guardLeaving(() => false);
  let moved = true;
  act(() => {
    moved = pushAddress("/admin/workflows/revive/");
  });
  expect(moved).toBe(false);
  expect(result.current.path).toBe("/admin/workflows/");
  release();
  act(() => {
    pushAddress("/admin/workflows/revive/");
  });
  expect(result.current.path).toBe("/admin/workflows/revive/");
});

it("going back past a refusing guard restores the address it left", () => {
  const { result } = renderHook(() => useAddress());
  act(() => {
    pushAddress("/admin/workflows/revive/edit/");
  });
  const release = guardLeaving(() => false);
  act(() => {
    window.history.replaceState(null, "", "/admin/workflows/revive/");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  expect(window.location.pathname).toBe("/admin/workflows/revive/edit/");
  expect(result.current.path).toBe("/admin/workflows/revive/edit/");
  release();
  act(() => {
    window.history.replaceState(null, "", "/admin/workflows/revive/");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  expect(result.current.path).toBe("/admin/workflows/revive/");
});
