import { act, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { servicesKey, servicesSchema, useServices } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { ProbeNowButton } from "./probe-now-button";

const services = servicesSchema.parse(apiSamples.services).services;

function Live() {
  const listed = useServices();
  const media = listed.data?.data.services[0];
  return media ? <ProbeNowButton service={media} /> : null;
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

it("asks for a probe, shows progress and polls until the result changes", async () => {
  const media = services[0];
  let checked = media.status.checked_at;
  const fetch = vi.fn(async (input: RequestInfo | URL) => {
    const path = String(input);
    if (path.endsWith("/probe")) {
      return new Response(null, { status: 202 });
    }
    return new Response(
      JSON.stringify({ services: [{ ...apiSamples.services.services[0], status: { ...apiSamples.services.services[0].status, checked_at: checked } }] }),
      { headers: { "Content-Type": "application/json" } },
    );
  });
  vi.stubGlobal("fetch", fetch);
  const client = testQueryClient();
  client.setQueryData(servicesKey, { data: { services }, revision: '"r"' });
  renderWithProviders(<Live />, client);
  await userEvent.click(await screen.findByRole("button", { name: "Check now" }));
  expect(await screen.findByRole("button", { name: "Checking…" })).toBeDisabled();
  expect(fetch).toHaveBeenCalledWith("/api/services/media/probe", expect.objectContaining({ method: "POST" }));
  checked = "2026-09-22T11:00:00Z";
  expect(await screen.findByRole("button", { name: "Check now" }, { timeout: 3000 })).toBeEnabled();
  expect(fetch.mock.calls.some((call) => String(call[0]) === "/api/services")).toBe(true);
});

it("gives up waiting after the timeout plus two seconds", async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  const media = services[0];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: RequestInfo | URL) =>
      String(input).endsWith("/probe")
        ? new Response(null, { status: 202 })
        : new Response(JSON.stringify(apiSamples.services), { headers: { "Content-Type": "application/json" } }),
    ),
  );
  renderWithProviders(<ProbeNowButton service={media} />);
  await userEvent.setup({ advanceTimers: vi.advanceTimersByTime }).click(screen.getByRole("button", { name: "Check now" }));
  expect(await screen.findByRole("button", { name: "Checking…" })).toBeDisabled();
  await act(async () => {
    vi.advanceTimersByTime((media.probe.timeout_seconds + 3) * 1000);
  });
  expect(await screen.findByRole("button", { name: "Check now" })).toBeEnabled();
});

it("tells the person to wait when asked too often", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("too many", { status: 429, headers: { "Retry-After": "4" } })),
  );
  renderWithProviders(<ProbeNowButton service={services[0]} />);
  await userEvent.click(screen.getByRole("button", { name: "Check now" }));
  expect(await screen.findByText("Too often: try again in 4 s")).toBeInTheDocument();
});

it("is not offered for a service whose probing is off", () => {
  renderWithProviders(<ProbeNowButton service={services[2]} />);
  expect(screen.getByRole("button", { name: "Check now" })).toBeDisabled();
  expect(screen.getByText("Checking is off in the service's settings")).toBeInTheDocument();
});
