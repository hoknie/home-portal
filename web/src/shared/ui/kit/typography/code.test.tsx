import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Code } from "./code";

it("sets inline code in the mono face on a tint", () => {
  render(<Code>home-portal.toml</Code>);
  expect(screen.getByText("home-portal.toml")).toHaveClass("font-mono", "bg-glass-tint");
});
