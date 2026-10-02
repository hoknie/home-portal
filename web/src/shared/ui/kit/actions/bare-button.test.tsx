import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { BareButton } from "./bare-button";

it("is a real button with no look of its own beyond the focus ring", async () => {
  const pressed = vi.fn();
  render(
    <BareButton className="rounded-full px-2" onClick={pressed}>
      chip
    </BareButton>,
  );
  const button = screen.getByRole("button", { name: "chip" });
  await userEvent.click(button);
  expect(pressed).toHaveBeenCalledOnce();
  expect(button).toHaveAttribute("type", "button");
  expect(button).toHaveClass("rounded-full", "focus-visible:ring-[3px]");
});
