import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { IconButton } from "./icon-button";

it("is named by its label and acts on a click", async () => {
  const pressed = vi.fn();
  render(
    <IconButton label="delete" onClick={pressed}>
      <svg />
    </IconButton>,
  );
  await userEvent.click(screen.getByRole("button", { name: "delete" }));
  expect(pressed).toHaveBeenCalledOnce();
  expect(screen.getByRole("button")).toHaveAttribute("data-variant", "ghost");
});
