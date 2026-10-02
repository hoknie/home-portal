import { fireEvent, render, renderHook, act, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { Splitter } from "./splitter";
import { useStoredSize } from "./use-stored-size";

afterEach(() => window.localStorage.clear());

it("drags and steps the size within its bounds, the other way when the panel is on the right", () => {
  const change = vi.fn();
  render(<Splitter label="Width of the tree" orientation="vertical" value={256} min={200} max={400} direction={-1} onChange={change} />);
  const handle = screen.getByRole("separator", { name: "Width of the tree" });
  fireEvent.pointerDown(handle, { pointerId: 1, clientX: 500 });
  fireEvent.pointerMove(handle, { pointerId: 1, clientX: 450 });
  expect(change).toHaveBeenLastCalledWith(306, false);
  fireEvent.pointerUp(handle, { pointerId: 1, clientX: 300 });
  expect(change).toHaveBeenLastCalledWith(400, true);
  fireEvent.keyDown(handle, { key: "ArrowLeft" });
  expect(change).toHaveBeenLastCalledWith(272, true);
  fireEvent.keyDown(handle, { key: "Home" });
  expect(change).toHaveBeenLastCalledWith(200, true);
});

it("remembers a size once a change is done", () => {
  const { result } = renderHook(() => useStoredSize("test.size", 300));
  expect(result.current[0]).toBe(300);
  act(() => result.current[1](320, false));
  expect(window.localStorage.getItem("test.size")).toBeNull();
  act(() => result.current[1](340, true));
  expect(window.localStorage.getItem("test.size")).toBe("340");
  expect(renderHook(() => useStoredSize("test.size", 300)).result.current[0]).toBe(340);
});
