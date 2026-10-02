import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { StatusDot } from "./status-dot";

it("draws the status colour, and is announced only with a label", () => {
  const { container } = render(
    <>
      <StatusDot tone="down" label="down" />
      <StatusDot tone="up" />
    </>,
  );
  expect(screen.getByRole("img", { name: "down" })).toHaveClass("bg-status-down");
  expect(container.querySelector("[data-tone='up']")).toHaveAttribute("aria-hidden", "true");
});
