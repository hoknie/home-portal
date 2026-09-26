import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { networkSchema } from "@/entities/network";
import { apiSamples } from "@/shared/api";
import { renderWithProviders } from "@/shared/lib/testing";

import { RestartPortalButton } from "./restart-portal-button";

afterEach(() => {
  vi.unstubAllGlobals();
});

it("restarts from the network notice after confirmation", async () => {
  const fetch = vi.fn(async () => new Response(null, { status: 202 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RestartPortalButton network={networkSchema.parse(apiSamples.network)} />);
  await userEvent.click(screen.getByRole("button", { name: "Restart now" }));
  await userEvent.click(screen.getByRole("button", { name: "Restart portal" }));
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/portal/restart", expect.objectContaining({ method: "POST" })));
  expect(await screen.findByText("The portal is restarting…")).toBeInTheDocument();
});
