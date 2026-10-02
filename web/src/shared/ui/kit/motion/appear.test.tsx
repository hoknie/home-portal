import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Appear } from "./appear";

it("fades its content in briefly", () => {
  render(<Appear>content</Appear>);
  expect(screen.getByText("content")).toHaveClass("animate-in", "fade-in-0", "duration-150");
});
