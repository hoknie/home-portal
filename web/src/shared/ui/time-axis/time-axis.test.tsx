import { render } from "@testing-library/react";
import { expect, it } from "vitest";

import { TimeAxis } from "./time-axis";

it("places each label at its share of the range and hides the axis from screen readers", () => {
  const from = new Date(2026, 8, 23, 0, 0).getTime();
  const to = from + 24 * 3_600_000;
  const { container } = render(<TimeAxis from={from} to={to} format={(at) => `${new Date(at).getHours()}h`} />);
  const axis = container.firstElementChild as HTMLElement;
  expect(axis).toHaveAttribute("aria-hidden", "true");
  const noon = [...axis.querySelectorAll<HTMLElement>("[data-tick]")].find((label) => label.textContent === "12h");
  expect(noon?.style.left).toBe("50%");
});
