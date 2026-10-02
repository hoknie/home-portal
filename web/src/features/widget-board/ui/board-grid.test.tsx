import { screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { renderWithProviders } from "@/shared/lib/testing";

import { BoardGrid } from "./board-grid";

const dashboard = dashboardSchema.parse(apiSamples.dashboard);
const services = servicesSchema.parse(apiSamples.services).services;

it("draws each section under its title with its widgets at their widths and heights", () => {
  const { container } = renderWithProviders(
    <BoardGrid sections={dashboard.sections} widgets={dashboard.widgets.filter((widget) => widget.type !== "weather")} services={services} />,
  );
  expect(screen.getByRole("heading", { level: 2, name: "Now" })).toBeInTheDocument();
  const media = container.querySelector('[data-section="media"]') as HTMLElement;
  expect(media).toHaveAttribute("aria-label", "Media");
  const widths = [...container.querySelectorAll("[data-width]")].map((element) => element.getAttribute("data-width"));
  expect(widths).toEqual(["8", "6", "5"]);
  const cells = container.querySelectorAll<HTMLElement>('[data-section="media"] [data-width]');
  expect([...cells].map((cell) => [cell.style.getPropertyValue("--span"), cell.style.getPropertyValue("--tablet-span")])).toEqual([
    ["6", "6"],
    ["5", "6"],
  ]);
  expect(media).toHaveAttribute("data-surface", "card");
  expect(media.querySelector(":scope > h2")).toBeNull();
});

it("places widgets in order on a grid of 8 px steps, a fixed height spanning twelve steps per 80 px row", () => {
  const sized = [
    { width: 3, height: 2 },
    { width: 3, height: 2 },
    { width: 6, height: 4 },
    { width: 6, height: 2 },
  ].map((size, index) => ({ ...dashboard.widgets[0], key: `w${index}`, title: `Box ${index}`, section: "now", ...size }));
  const { container } = renderWithProviders(<BoardGrid sections={dashboard.sections} widgets={sized} services={services} />);
  const grid = container.querySelector('[data-section="now"] > div') as HTMLElement;
  expect(grid.className).toContain("auto-rows-[8px]");
  expect(grid.className).not.toContain("grid-flow-dense");
  const cells = [...grid.children] as HTMLElement[];
  expect(cells.map((cell) => [cell.style.getPropertyValue("--span"), cell.style.getPropertyValue("--rows")])).toEqual([
    ["3", "24"],
    ["3", "24"],
    ["6", "48"],
    ["6", "24"],
  ]);
});

it("draws no title for a section whose widgets are all hidden", () => {
  renderWithProviders(
    <BoardGrid sections={dashboard.sections} widgets={dashboard.widgets.filter((widget) => widget.section === "media")} services={services} />,
  );
  expect(screen.queryByRole("heading", { level: 2, name: "Now" })).not.toBeInTheDocument();
});
