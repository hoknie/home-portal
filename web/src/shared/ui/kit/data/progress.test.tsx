import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Progress } from "./progress";

it("fills to its percent, clamped, in its tone", () => {
  render(<Progress percent={140} tone="alarm" label="disk" role="meter" />);
  const bar = screen.getByRole("meter", { name: "disk" });
  expect(bar).toHaveAttribute("aria-valuenow", "100");
  expect(bar.firstElementChild).toHaveClass("bg-status-down");
  expect(bar.firstElementChild).toHaveStyle({ width: "100%" });
});
