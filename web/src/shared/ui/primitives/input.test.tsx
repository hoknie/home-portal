import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Input } from "./input";

it("accepts typing and marks itself invalid when asked", async () => {
  render(<Input aria-label="field" aria-invalid />);
  const field = screen.getByLabelText("field");
  await userEvent.type(field, "abc");
  expect(field).toHaveValue("abc");
  expect(field).toHaveAttribute("aria-invalid", "true");
  expect(field).toHaveClass("bg-glass-tint");
});
