import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { HistoryChart, type HistoryChartProps } from "./history-chart";
import { type HistoryPoint, bandsOf, intervalsOf, linesOf, smoothPath } from "./plotting";

const HOUR = 3_600_000;
const FROM = Date.UTC(2026, 8, 22, 0, 0);
const TO = FROM + 24 * HOUR;
const OUTAGE = FROM + 3 * HOUR;

const STEP = 15 * 60_000;

function interval(at: number): HistoryPoint {
  if (at >= OUTAGE && at < OUTAGE + 30 * 60_000) {
    return { at, average: null, minimum: null, maximum: null, state: "down" };
  }
  const peak = at === FROM + 14 * HOUR;
  return { at, average: 25 + ((at / STEP) % 5) * 2, minimum: 20, maximum: peak ? 430 : 40, state: "up" };
}

const points = Array.from({ length: 96 }, (_, index) => interval(FROM + index * STEP));

const props: HistoryChartProps = {
  points,
  step: STEP,
  from: FROM,
  to: TO,
  title: "History over 24 hours",
  summary: "Mostly up, peak 430 ms",
  empty: "No data",
  legend: { average: "Average latency", range: "From the minimum to the maximum", states: { unknown: "Unknown", up: "Up", degraded: "Slow", down: "Down", unreadable: "Unreadable" } },
  formatTime: (at) => `${new Date(at).getUTCHours()}h`,
  formatValue: (value) => `${value} ms`,
  hintOf: (interval) => ({
    time: `${new Date(interval.at).getUTCHours()}:${String(new Date(interval.at).getUTCMinutes()).padStart(2, "0")}`,
    state: interval.state ?? "none",
    average: interval.average === null ? "no latency" : `avg ${Math.round(interval.average)} ms`,
    maximum: interval.maximum === null ? "" : `max ${interval.maximum} ms`,
  }),
};

it("each interval the portal sends lasts one step, and the last one ends at the end of the period", () => {
  const intervals = intervalsOf(points, STEP, TO - 5 * 60_000);
  expect(intervals).toHaveLength(96);
  expect(intervals[0].until - intervals[0].at).toBe(STEP);
  expect(intervals.at(-1)?.until).toBe(TO - 5 * 60_000);
});

it("the line is a smooth curve broken at an outage, never drawing zero", () => {
  const intervals = intervalsOf(points, STEP, TO);
  const lines = linesOf(intervals, FROM, TO, 600);
  expect(lines).toHaveLength(2);
  expect(lines[0].line).toMatch(/^M[\d.]+,[\d.]+ C/);
  const { container } = render(<HistoryChart {...props} />);
  expect(container.querySelectorAll("[data-segment]")).toHaveLength(2);
  expect(container.querySelectorAll("[data-range]")).toHaveLength(2);
});

it("a smooth curve through rising values never dips below or overshoots its neighbours", () => {
  const path = smoothPath([
    { x: 0, y: 100 },
    { x: 10, y: 90 },
    { x: 20, y: 10 },
    { x: 30, y: 9 },
  ]);
  const ys = [...path.matchAll(/,([\d.]+)/g)].map((match) => Number(match[1]));
  expect(Math.max(...ys)).toBeLessThanOrEqual(100);
  expect(Math.min(...ys)).toBeGreaterThanOrEqual(9);
});

it("colours the state band by the worst state of each interval over the same time axis", () => {
  const bands = bandsOf(intervalsOf(points, STEP, TO));
  expect(bands.map((band) => band.state)).toEqual(["up", "down", "up"]);
  expect(bands[1]).toMatchObject({ start: OUTAGE, end: OUTAGE + 30 * 60_000 });
  const { container } = render(<HistoryChart {...props} />);
  expect(container.querySelectorAll('[data-band="down"]')).toHaveLength(1);
  expect(screen.getByRole("list", { name: "History over 24 hours" })).toHaveTextContent("Down");
  expect(container.querySelectorAll("[data-under]")).toHaveLength(2);
  expect(screen.getByRole("list", { name: "History over 24 hours" })).not.toHaveTextContent("Unreadable");
});

it("labels a millisecond axis in round steps reaching the highest maximum, and draws one time axis", () => {
  const { container } = render(<HistoryChart {...props} />);
  expect([...container.querySelectorAll("[data-value-tick]")].map((label) => label.textContent).at(-1)).toBe("600 ms");
  expect(container.querySelectorAll("[data-tick]").length).toBeGreaterThanOrEqual(2);
});

it("moving along the chart with the keyboard shows the hint of each interval and announces it", () => {
  const { container } = render(<HistoryChart {...props} />);
  const chart = screen.getByRole("group", { name: "History over 24 hours" });
  expect(chart).toHaveAccessibleDescription("Mostly up, peak 430 ms");
  chart.focus();
  fireEvent.keyDown(chart, { key: "Home" });
  for (let step = 0; step < 12; step += 1) {
    fireEvent.keyDown(chart, { key: "ArrowRight" });
  }
  expect(container.querySelector("[data-hint]")).toHaveTextContent("3:00downno latency");
  expect(container.querySelector("[aria-live]")).toHaveTextContent("3:00: down, no latency");
  expect(container.querySelector("[data-marker]")).not.toBeNull();
});

it("pointing at the peak shows its interval and maximum", () => {
  const { container } = render(<HistoryChart {...props} />);
  const chart = screen.getByRole("group", { name: "History over 24 hours" });
  chart.getBoundingClientRect = () => ({ left: 0, width: 960, top: 0, height: 100, right: 960, bottom: 100, x: 0, y: 0, toJSON: () => ({}) });
  fireEvent.pointerMove(chart, { clientX: 14 * 40 + 5 });
  expect(container.querySelector("[data-hint]")).toHaveTextContent("14:00");
  expect(container.querySelector("[data-hint]")).toHaveTextContent(/max 4\d\d ms/);
  fireEvent.pointerLeave(chart);
  expect(container.querySelector("[data-hint]")).toBeNull();
});

it("says there is no data instead of drawing an empty axis", () => {
  render(<HistoryChart {...props} points={[]} />);
  expect(screen.getByText("No data")).toBeInTheDocument();
});
