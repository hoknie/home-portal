import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { LatencyChart } from "../latency-chart";
import { UptimeStrip } from "./uptime-strip";

const FROM = new Date(2026, 8, 1).getTime();
const TO = new Date(2026, 8, 30).getTime();
const formatTime = (at: number) => `${new Date(at).getDate()}.09`;

it("draws one slot per period, colours it by state and leaves a period without probes hollow", () => {
  const { container } = render(
    <UptimeStrip
      title="Availability"
      from={FROM}
      to={TO}
      formatTime={formatTime}
      slots={[
        { key: "1", state: "up", label: "10:00 — up" },
        { key: "2", state: null, label: "11:00 — no data" },
        { key: "3", state: "down", label: "12:00 — down" },
      ]}
    />,
  );
  expect(screen.getByRole("img", { name: "Availability" })).toBeInTheDocument();
  const states = [...container.querySelectorAll("[data-state]")].map((slot) => slot.getAttribute("data-state"));
  expect(states).toEqual(["up", "none", "down"]);
  expect(container.querySelector('[data-state="none"]')?.className).toContain("border-dashed");
  expect(container.querySelector('[data-state="down"]')).toHaveAttribute("title", "12:00 — down");
});

it("marks the same times at the same places as the latency chart of the range", () => {
  const marks = (container: HTMLElement) =>
    [...container.querySelectorAll<HTMLElement>("[data-tick]")].map((tick) => [tick.textContent, tick.style.left]);
  const strip = render(<UptimeStrip title="Availability" from={FROM} to={TO} formatTime={formatTime} slots={[]} />);
  const chart = render(
    <LatencyChart
      title="Latency"
      empty="—"
      from={FROM}
      to={TO}
      formatTime={formatTime}
      formatValue={String}
      samples={[{ at: new Date(FROM).toISOString(), value: 5, failed: false, label: "a" }]}
    />,
  );
  expect(marks(strip.container).length).toBeGreaterThanOrEqual(2);
  expect(marks(strip.container)).toEqual(marks(chart.container));
});
