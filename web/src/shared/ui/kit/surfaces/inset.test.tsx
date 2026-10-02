import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Inset } from "./inset";

it("draws a well on the inset surface", () => {
  render(<Inset>well</Inset>);
  expect(screen.getByText("well")).toHaveClass("surface-inset", "rounded-lg");
});
