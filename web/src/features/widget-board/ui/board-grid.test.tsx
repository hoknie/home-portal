import { screen, within } from "@testing-library/react";
import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { renderWithProviders } from "@/shared/lib/testing";

import { BoardGrid } from "./board-grid";

const dashboard = dashboardSchema.parse(apiSamples.dashboard);
const services = servicesSchema.parse(apiSamples.services).services;

it("draws each section under its title with its widgets at their sizes", () => {
  const { container } = renderWithProviders(
    <BoardGrid sections={dashboard.sections} widgets={dashboard.widgets.filter((widget) => widget.type !== "weather")} services={services} />,
  );
  expect(screen.getByRole("heading", { level: 2, name: "Now" })).toBeInTheDocument();
  const media = screen.getByRole("region", { name: "Media" });
  expect(within(media).getAllByRole("heading", { level: 2 })[0]).toHaveTextContent("Media");
  const sizes = [...container.querySelectorAll("[data-size]")].map((element) => element.getAttribute("data-size"));
  expect(sizes).toEqual(["two-thirds", "half", "half"]);
  const halves = container.querySelectorAll('[data-section="media"] [data-size="half"]');
  expect(halves).toHaveLength(2);
  for (const half of halves) {
    expect(half.className).toContain("sm:col-span-6");
  }
});

it("draws no title for a section whose widgets are all hidden", () => {
  renderWithProviders(
    <BoardGrid sections={dashboard.sections} widgets={dashboard.widgets.filter((widget) => widget.section === "media")} services={services} />,
  );
  expect(screen.queryByRole("heading", { level: 2, name: "Now" })).not.toBeInTheDocument();
});
