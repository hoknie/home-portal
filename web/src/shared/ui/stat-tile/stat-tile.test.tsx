import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { StatTile } from "./stat-tile";

it("puts a number under its label", () => {
  render(<StatTile label="Всего" value={3} hint="сервисов" />);
  expect(screen.getByText("Всего").nextSibling).toHaveTextContent("3");
  expect(screen.getByText("сервисов")).toBeInTheDocument();
});
