import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { LatencyChart, plot } from "./latency-chart";

const at = (minute: number) => new Date(Date.UTC(2026, 8, 22, 10, minute)).toISOString();
const FROM = Date.parse(at(0));
const TO = Date.parse(at(60));
const axes = { from: FROM, to: TO, formatTime: (moment: number) => `t${new Date(moment).getUTCMinutes()}`, formatValue: (value: number) => `${value} ms` };

const samples = [
  { at: at(0), value: 10, failed: false, label: "a" },
  { at: at(1), value: 12, failed: false, label: "b" },
  { at: at(2), value: null, failed: true, label: "c" },
  { at: at(3), value: 11, failed: false, label: "d" },
  { at: at(4), value: 13, failed: false, label: "e" },
  { at: at(30), value: 430, failed: false, label: "f" },
];

it("breaks the line where a probe failed and where no probe ran, never drawing zero", () => {
  const { segments } = plot(samples, FROM, TO, 600);
  expect(segments).toHaveLength(3);
  const { container } = render(<LatencyChart samples={samples} title="Latency" empty="No data" {...axes} />);
  expect(screen.getByRole("img", { name: "Latency" })).toBeInTheDocument();
  expect(container.querySelectorAll("[data-segment]")).toHaveLength(3);
  expect(container.querySelectorAll("[data-failed]")).toHaveLength(1);
});

it("spreads the samples over the whole range, not from the first to the last", () => {
  const { points } = plot(samples, FROM, TO, 600);
  expect(points[0].x).toBe(0);
  expect(points[5].x).toBeCloseTo(300);
});

it("labels a millisecond axis in round steps reaching the highest value", () => {
  const { container } = render(<LatencyChart samples={samples} title="Latency" empty="No data" {...axes} />);
  const labels = [...container.querySelectorAll("[data-value-tick]")].map((label) => label.textContent);
  expect(labels).toEqual(["0 ms", "200 ms", "400 ms", "600 ms"]);
  expect(container.querySelector('[data-value-tick="0"]')?.closest("[aria-hidden]")).not.toBeNull();
});

it("draws a time axis under the chart", () => {
  const { container } = render(<LatencyChart samples={samples} title="Latency" empty="No data" {...axes} />);
  expect(container.querySelectorAll("[data-tick]").length).toBeGreaterThanOrEqual(2);
});

it("says there is no data instead of drawing an empty axis", () => {
  render(<LatencyChart samples={[]} title="Latency" empty="No data" {...axes} />);
  expect(screen.getByText("No data")).toBeInTheDocument();
});
