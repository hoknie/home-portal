import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { Button } from "./button";

it("is a button that reports clicks and can be disabled", async () => {
  const click = vi.fn();
  const { rerender } = render(<Button onClick={click}>save</Button>);
  await userEvent.click(screen.getByRole("button", { name: "save" }));
  expect(click).toHaveBeenCalledOnce();
  rerender(<Button disabled onClick={click}>save</Button>);
  expect(screen.getByRole("button", { name: "save" })).toBeDisabled();
});

it("keeps the primary action opaque and tints the quieter variants", () => {
  render(
    <>
      <Button>primary</Button>
      <Button variant="outline">outline</Button>
      <Button variant="secondary">secondary</Button>
    </>,
  );
  expect(screen.getByRole("button", { name: "primary" })).toHaveClass("bg-primary");
  expect(screen.getByRole("button", { name: "primary" })).not.toHaveClass("bg-glass-tint");
  expect(screen.getByRole("button", { name: "outline" })).toHaveClass("bg-glass-tint");
  expect(screen.getByRole("button", { name: "secondary" })).toHaveClass("bg-glass-tint");
});
