import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { MetricBar, toneOf } from "./metric-bar";

it("shows how full something is, as a meter and as text", () => {
  render(<MetricBar label="Memory" value="12 of 24 GB" percent={50} />);
  const meter = screen.getByRole("meter", { name: "Memory" });
  expect(meter).toHaveAttribute("aria-valuenow", "50");
  expect(screen.getByText("12 of 24 GB")).toBeInTheDocument();
});

it("keeps the bar inside its track whatever the number says", () => {
  render(<MetricBar label="Disk" value="full" percent={140} />);
  expect(screen.getByRole("meter", { name: "Disk" })).toHaveAttribute("aria-valuenow", "100");
});

it("turns warning and then alarm as it fills", () => {
  expect(toneOf(10)).toBe("neutral");
  expect(toneOf(80)).toBe("warning");
  expect(toneOf(95)).toBe("alarm");
});
