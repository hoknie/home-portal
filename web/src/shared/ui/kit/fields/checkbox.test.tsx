import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { Checkbox } from "./checkbox";

it("toggles and reports its state", async () => {
  const changed = vi.fn();
  render(<Checkbox aria-label="public" onCheckedChange={changed} />);
  const box = screen.getByRole("checkbox", { name: "public" });
  await userEvent.click(box);
  expect(changed).toHaveBeenCalledWith(true);
  expect(box).toHaveAttribute("data-state", "checked");
});

it("shows a mixed state for a partly chosen group", () => {
  render(<Checkbox aria-label="every action" checked="indeterminate" />);
  expect(screen.getByRole("checkbox", { name: "every action" })).toHaveAttribute("aria-checked", "mixed");
});
