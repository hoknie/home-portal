import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { metricsSchema, widgetDataSchema } from "@/entities/widget";
import { apiSamples } from "@/shared/api";
import { TestIntl } from "@/shared/i18n";

import { HostMetricsWidget, gibibytes, hours } from "./host-metrics-widget";

const data = metricsSchema.parse(widgetDataSchema.parse(apiSamples.widgetHostMetrics).data);

function renderWidget() {
  render(
    <TestIntl>
      <HostMetricsWidget data={data} />
    </TestIntl>,
  );
}

it("shows the processor, the memory and every disk as a meter", () => {
  renderWidget();
  const meters = screen.getAllByRole("meter").map((meter) => meter.getAttribute("aria-label"));
  expect(meters).toEqual(["CPU", "Memory", "/", "/media"]);
  expect(screen.getByRole("meter", { name: "CPU" })).toHaveAttribute("aria-valuenow", "18");
});

it("counts memory and disks in gibibytes", () => {
  renderWidget();
  expect(screen.getByText("6.4 of 16 GiB")).toBeInTheDocument();
  expect(screen.getByText("340.4 of 460.4 GiB")).toBeInTheDocument();
});

it("names the host and its uptime in whole hours", () => {
  renderWidget();
  expect(screen.getByText("box")).toBeInTheDocument();
  expect(screen.getByText("26 h")).toBeInTheDocument();
  expect(hours(93_600)).toBe(26);
  expect(gibibytes(1024 ** 3 * 2.5)).toBe(2.5);
});
