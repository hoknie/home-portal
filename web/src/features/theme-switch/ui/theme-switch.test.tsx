import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "@/shared/ui/primitives";

import { ThemeSwitch } from "./theme-switch";

const setTheme = vi.fn();

vi.mock("next-themes", () => ({ useTheme: () => ({ theme: "system", setTheme }) }));

it("offers light, dark and system and applies the choice", async () => {
  renderWithProviders(
    <DropdownMenu>
      <DropdownMenuTrigger>menu</DropdownMenuTrigger>
      <DropdownMenuContent>
        <ThemeSwitch />
      </DropdownMenuContent>
    </DropdownMenu>,
  );
  await userEvent.click(screen.getByText("menu"));
  expect(screen.getByRole("menuitemradio", { name: "System" })).toHaveAttribute("aria-checked", "true");
  await userEvent.click(screen.getByRole("menuitemradio", { name: "Dark" }));
  expect(setTheme).toHaveBeenCalledWith("dark");
});
