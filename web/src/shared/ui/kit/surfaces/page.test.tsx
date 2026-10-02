import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Page } from "./page";

it("stacks a page's sections with one gap", () => {
  render(<Page>content</Page>);
  expect(screen.getByText("content")).toHaveClass("grid", "gap-6");
});
