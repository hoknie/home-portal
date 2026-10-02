import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Textarea } from "./textarea";

it("takes several lines in the field style", async () => {
  render(<Textarea aria-label="notes" />);
  const field = screen.getByLabelText("notes");
  await userEvent.type(field, "one{enter}two");
  expect(field).toHaveValue("one\ntwo");
  expect(field).toHaveClass("bg-glass-tint", "border-input");
});
