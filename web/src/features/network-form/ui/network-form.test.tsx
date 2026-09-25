import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { networkSchema } from "@/entities/network";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { NetworkForm } from "./network-form";

const network = networkSchema.parse(apiSamples.network);

afterEach(() => {
  vi.unstubAllGlobals();
});

it("sends the changed port with the revision it loaded", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.network, { headers: { ETag: '"r2"' } }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<NetworkForm configured={network.configured} revision='"r1"' />);
  const port = screen.getByLabelText("Порт");
  await userEvent.clear(port);
  await userEvent.type(port, "9191");
  await userEvent.click(screen.getByRole("button", { name: "Сохранить" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(JSON.parse(init.body as string)).toMatchObject({ port: 9191, trusted_proxies: ["10.0.0.0/8"] });
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
});

it("maps a server error on one proxy to the proxies field", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ errors: [{ field: "trusted_proxies[0]", message: "bad range" }] }, { status: 422 })));
  renderWithProviders(<NetworkForm configured={network.configured} revision='"r1"' />);
  const port = screen.getByLabelText("Порт");
  await userEvent.clear(port);
  await userEvent.type(port, "9191");
  await userEvent.click(screen.getByRole("button", { name: "Сохранить" }));
  expect(await screen.findByText("bad range")).toBeInTheDocument();
});
