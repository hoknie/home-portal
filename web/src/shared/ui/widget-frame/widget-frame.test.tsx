import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { type Appearance, DEFAULT_APPEARANCE, SURFACES } from "@/shared/api";
import { TestIntl } from "@/shared/i18n";

import { WidgetFrame, WidgetPanel, WidgetProblem } from "./widget-frame";

it("names the widget and marks data that is no longer current", () => {
  render(
    <TestIntl>
      <WidgetFrame title="Weather" stale problem="upstream is away">
        <p>12°</p>
      </WidgetFrame>
    </TestIntl>,
  );
  expect(screen.getByRole("heading", { level: 2, name: "Weather" })).toBeInTheDocument();
  expect(screen.getByText(/out of date/)).toHaveAttribute("data-stale", "true");
  expect(screen.getByText("12°")).toBeInTheDocument();
});

it("shows a skeleton instead of its content while loading", () => {
  const { container } = render(
    <TestIntl>
      <WidgetFrame title="Weather" loading>
        <p>12°</p>
      </WidgetFrame>
    </TestIntl>,
  );
  expect(container.querySelector("[aria-busy=true]")).not.toBeNull();
  expect(screen.queryByText("12°")).not.toBeInTheDocument();
});

it("explains a failure inside its own frame", () => {
  render(
    <TestIntl>
      <WidgetProblem message="the calendar answered 500" />
    </TestIntl>,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("the calendar answered 500");
});

const look = (patch: Partial<Appearance>): Appearance => ({ ...DEFAULT_APPEARANCE, ...patch });

it("draws each surface from the theme, and a plain one without a card", () => {
  const surfaces = SURFACES.map((surface) => {
    const { container, unmount } = render(
      <TestIntl>
        <WidgetFrame title="Disks" appearance={look({ surface })}>
          <p>120 GB</p>
        </WidgetFrame>
      </TestIntl>,
    );
    const frame = container.querySelector("section") as HTMLElement;
    const drawn = [frame.getAttribute("data-surface"), frame.className.includes("surface-panel"), frame.className.includes("widget-tint"), frame.className.includes("border")];
    unmount();
    return drawn;
  });
  expect(surfaces).toEqual([
    ["card", true, false, false],
    ["plain", false, false, false],
    ["tinted", false, true, false],
    ["outline", false, false, true],
  ]);
});

it("a hidden title stays the widget's accessible name, and an accent marks the title", () => {
  render(
    <TestIntl>
      <WidgetFrame title="Disks" appearance={look({ title: "hidden", accent: "green", align: "center" })}>
        <p>120 GB</p>
      </WidgetFrame>
    </TestIntl>,
  );
  const region = screen.getByRole("region", { name: "Disks" });
  expect(region).toHaveAttribute("data-accent", "green");
  expect(region).toHaveAttribute("data-align", "center");
  expect(screen.getByRole("heading", { name: "Disks" }).parentElement?.className).toContain("sr-only");
});

it("a panel inside a card adds no second card, and draws one on a plain surface", () => {
  const panelIn = (surface: Appearance["surface"]) => {
    const { container, unmount } = render(
      <TestIntl>
        <WidgetFrame title="Disks" appearance={look({ surface })}>
          <WidgetPanel className="grid">x</WidgetPanel>
        </WidgetFrame>
      </TestIntl>,
    );
    const panel = container.querySelector("section .grid") as HTMLElement;
    const card = panel.className.includes("surface-panel");
    unmount();
    return card;
  };
  expect(panelIn("card")).toBe(false);
  expect(panelIn("plain")).toBe(true);
});

it("a fixed height lets the content scroll inside the frame", () => {
  const { container } = render(
    <TestIntl>
      <WidgetFrame title="Disks" fill>
        <p>120 GB</p>
      </WidgetFrame>
    </TestIntl>,
  );
  expect(container.querySelector("section")?.className).toContain("h-full");
  expect(screen.getByText("120 GB").parentElement?.className).toContain("overflow-auto");
});

it("aligns its title and content to the end", () => {
  render(
    <TestIntl>
      <WidgetFrame title="Weather" appearance={look({ align: "end" })}>
        <p>20 °C</p>
      </WidgetFrame>
    </TestIntl>,
  );
  const region = screen.getByRole("region", { name: "Weather" });
  expect(region).toHaveAttribute("data-align", "end");
  expect(region).toHaveClass("items-end", "text-right");
});
