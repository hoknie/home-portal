import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "@/shared/ui/primitives";

import { RestartPortalItem } from "./restart-portal-item";

const toast = vi.hoisted(() => ({ success: vi.fn(), error: vi.fn() }));
vi.mock("sonner", () => ({ toast }));

afterEach(() => {
  vi.unstubAllGlobals();
  toast.error.mockReset();
  document.body.style.pointerEvents = "";
});

function renderMenu() {
  renderWithProviders(
    <DropdownMenu>
      <DropdownMenuTrigger>menu</DropdownMenuTrigger>
      <DropdownMenuContent>
        <RestartPortalItem />
      </DropdownMenuContent>
    </DropdownMenu>,
  );
}

async function confirmRestart() {
  screen.getByText("menu").focus();
  await userEvent.keyboard("{Enter}");
  await userEvent.click(await screen.findByRole("menuitem", { name: "Restart portal" }));
  expect(screen.getByText("Restart the portal?")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Restart portal" }));
}

it("asks first, then posts the restart and covers the page while the portal comes back", async () => {
  const fetch = vi.fn(async (path: string) => (path === "/health" ? new Response("ok") : new Response(null, { status: 202 })));
  vi.stubGlobal("fetch", fetch);
  renderMenu();
  await confirmRestart();
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/portal/restart", expect.objectContaining({ method: "POST" })));
  expect(await screen.findByText("The portal is restarting…")).toBeInTheDocument();
});

it("a failed request says so and leaves the page as it is", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("boom", { status: 500 })));
  renderMenu();
  await confirmRestart();
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith("The portal could not be restarted"));
  expect(screen.queryByText("The portal is restarting…")).toBeNull();
});
