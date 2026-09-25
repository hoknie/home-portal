import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "@/shared/ui/primitives";

import { SignOutItem } from "./sign-out-item";

afterEach(() => {
  vi.unstubAllGlobals();
});

it("ends the session and returns to the home page", async () => {
  const fetch = vi.fn(async () => new Response(null, { status: 204 }));
  const assign = vi.fn();
  vi.stubGlobal("fetch", fetch);
  Object.defineProperty(window, "location", { value: { ...window.location, assign }, writable: true });
  renderWithProviders(
    <DropdownMenu>
      <DropdownMenuTrigger>menu</DropdownMenuTrigger>
      <DropdownMenuContent>
        <SignOutItem />
      </DropdownMenuContent>
    </DropdownMenu>,
  );
  await userEvent.click(screen.getByText("menu"));
  await userEvent.click(screen.getByRole("menuitem", { name: "Выйти" }));
  await waitFor(() => expect(assign).toHaveBeenCalledWith("/"));
  expect(fetch).toHaveBeenCalledWith("/api/session", expect.objectContaining({ method: "DELETE" }));
});
