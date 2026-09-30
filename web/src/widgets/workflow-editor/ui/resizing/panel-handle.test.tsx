import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { PanelHandle } from "./panel-handle";
import { PANEL_KEY } from "./panel-width";
import { rereadPanelWidth } from "./use-panel-width";

beforeEach(() => {
  window.localStorage.clear();
  rereadPanelWidth();
});

function open() {
  render(
    <TestIntl>
      <div data-testid="row">
        <PanelHandle />
      </div>
    </TestIntl>,
  );
  Object.defineProperty(screen.getByTestId("row"), "offsetWidth", { configurable: true, value: 1000 });
  return screen.getByRole("separator", { name: "Width of the side panel" });
}

it("the handle is a vertical separator that announces the panel's width", () => {
  const handle = open();
  expect(handle).toHaveAttribute("aria-orientation", "vertical");
  expect(handle).toHaveAttribute("aria-valuenow", "416");
  expect(handle).toHaveAttribute("aria-valuetext", "416 px");
  expect(handle).toHaveAttribute("tabindex", "0");
});

it("arrows step the width, Shift steps further, and Home and End reach the limits", () => {
  const handle = open();
  fireEvent.keyDown(handle, { key: "ArrowLeft" });
  expect(handle).toHaveAttribute("aria-valuenow", "432");
  fireEvent.keyDown(handle, { key: "ArrowRight", shiftKey: true });
  expect(handle).toHaveAttribute("aria-valuenow", "368");
  fireEvent.keyDown(handle, { key: "End" });
  expect(handle).toHaveAttribute("aria-valuenow", "700");
  expect(handle).toHaveAttribute("aria-valuemax", "700");
  fireEvent.keyDown(handle, { key: "Home" });
  expect(handle).toHaveAttribute("aria-valuenow", "320");
  expect(window.localStorage.getItem(PANEL_KEY)).toBe("320");
});

it("dragging left widens the panel and is stored when it ends", () => {
  const handle = open();
  fireEvent.pointerDown(handle, { clientX: 600, pointerId: 1 });
  fireEvent.pointerMove(handle, { clientX: 400, pointerId: 1 });
  expect(handle).toHaveAttribute("aria-valuenow", "616");
  expect(window.localStorage.getItem(PANEL_KEY)).toBeNull();
  fireEvent.pointerUp(handle, { clientX: 400, pointerId: 1 });
  expect(window.localStorage.getItem(PANEL_KEY)).toBe("616");
});

it("a double click resets the width", () => {
  const handle = open();
  fireEvent.keyDown(handle, { key: "End" });
  fireEvent.doubleClick(handle);
  expect(handle).toHaveAttribute("aria-valuenow", "416");
  expect(window.localStorage.getItem(PANEL_KEY)).toBe("416");
});
