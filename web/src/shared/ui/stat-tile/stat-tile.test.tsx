import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { StatTile } from "./stat-tile";

it("puts a number under its label", () => {
  render(<StatTile label="Total" value={3} hint="services" />);
  expect(screen.getByText("Total").nextSibling).toHaveTextContent("3");
  expect(screen.getByText("services")).toBeInTheDocument();
});
