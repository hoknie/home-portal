import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./dropdown-menu";

it("shows items when opened and reports a choice", async () => {
  const choose = vi.fn();
  render(
    <DropdownMenu>
      <DropdownMenuTrigger>menu</DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem onSelect={choose}>first</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>,
  );
  await userEvent.click(screen.getByText("menu"));
  expect(screen.getByRole("menu")).toHaveClass("glass-overlay");
  await userEvent.click(screen.getByRole("menuitem", { name: "first" }));
  expect(choose).toHaveBeenCalledOnce();
});
