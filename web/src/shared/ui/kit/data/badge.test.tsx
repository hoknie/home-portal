import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Badge } from "./badge";

it("renders its content with the chosen variant", () => {
  render(<Badge variant="secondary">beta</Badge>);
  expect(screen.getByText("beta")).toHaveAttribute("data-slot", "badge");
});
