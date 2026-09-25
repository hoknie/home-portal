import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { MetricBar, toneOf } from "./metric-bar";

it("shows how full something is, as a meter and as text", () => {
  render(<MetricBar label="Память" value="12 из 24 ГБ" percent={50} />);
  const meter = screen.getByRole("meter", { name: "Память" });
  expect(meter).toHaveAttribute("aria-valuenow", "50");
  expect(screen.getByText("12 из 24 ГБ")).toBeInTheDocument();
});

it("keeps the bar inside its track whatever the number says", () => {
  render(<MetricBar label="Диск" value="полон" percent={140} />);
  expect(screen.getByRole("meter", { name: "Диск" })).toHaveAttribute("aria-valuenow", "100");
});

it("turns warning and then alarm as it fills", () => {
  expect(toneOf(10)).toBe("neutral");
  expect(toneOf(80)).toBe("warning");
  expect(toneOf(95)).toBe("alarm");
});
