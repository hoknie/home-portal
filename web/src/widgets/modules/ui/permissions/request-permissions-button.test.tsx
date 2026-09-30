import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { RequestPermissionsButton } from "./request-permissions-button";

afterEach(() => {
  vi.unstubAllGlobals();
});

function answering(status: number, body = "") {
  const fetch = vi.fn(async () => new Response(body, { status }));
  vi.stubGlobal("fetch", fetch);
  return fetch;
}

it("asks the portal again with a POST", async () => {
  const fetch = answering(202);
  renderWithProviders(<RequestPermissionsButton />);
  await userEvent.click(screen.getByRole("button", { name: "Ask again" }));
  expect(fetch).toHaveBeenCalledWith("/api/permissions/request", expect.objectContaining({ method: "POST" }));
  expect(screen.queryByRole("alert")).toBeNull();
});

it("a request already running says to wait for the answers", async () => {
  answering(409, "permissions are being asked for already");
  renderWithProviders(<RequestPermissionsButton />);
  await userEvent.click(screen.getByRole("button", { name: "Ask again" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("The portal is already asking; wait for the answers on the Mac.");
});

it("a request from outside says it works only from inside", async () => {
  answering(403, "permissions can be asked for only from inside your environments");
  renderWithProviders(<RequestPermissionsButton />);
  await userEvent.click(screen.getByRole("button", { name: "Ask again" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Permissions can be asked for only from inside your networks.");
});
