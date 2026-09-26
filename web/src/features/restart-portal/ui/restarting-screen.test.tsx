import { act, fireEvent, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { GIVE_UP_MILLISECONDS, POLL_MILLISECONDS, RestartingScreen } from "./restarting-screen";

const ORIGIN = "http://nas.lan:8080";
const reload = vi.fn();
const assign = vi.fn();
const original = window.location;

beforeEach(() => {
  vi.useFakeTimers();
  Object.defineProperty(window, "location", {
    value: { origin: ORIGIN, pathname: "/admin/network/", search: "?tab=1", reload, assign },
    writable: true,
  });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  reload.mockReset();
  assign.mockReset();
  Object.defineProperty(window, "location", { value: original, writable: true });
});

async function tick(times = 1) {
  for (let index = 0; index < times; index += 1) {
    await act(async () => {
      await vi.advanceTimersByTimeAsync(POLL_MILLISECONDS);
    });
  }
}

it("reloads once the portal went away and answers again", async () => {
  const fetch = vi
    .fn()
    .mockRejectedValueOnce(new TypeError("down"))
    .mockResolvedValue(new Response("ok", { status: 200 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RestartingScreen target={ORIGIN} />);
  expect(screen.getByText("The portal is restarting…")).toBeInTheDocument();
  await tick();
  expect(reload).not.toHaveBeenCalled();
  await tick();
  expect(reload).toHaveBeenCalledTimes(1);
  expect(fetch).toHaveBeenCalledWith(`${ORIGIN}/health`, expect.objectContaining({ cache: "no-store" }));
});

it("goes to the new address with the same page once it answers", async () => {
  const fetch = vi.fn().mockResolvedValue(new Response(null, { status: 200 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RestartingScreen target="http://nas.lan:9090" />);
  await tick();
  expect(fetch).toHaveBeenCalledWith("http://nas.lan:9090/health", expect.objectContaining({ mode: "no-cors" }));
  expect(assign).toHaveBeenCalledWith("http://nas.lan:9090/admin/network/?tab=1");
});

it("says the portal did not come back after a minute and tries again on demand", async () => {
  const fetch = vi.fn().mockRejectedValue(new TypeError("down"));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RestartingScreen target={ORIGIN} />);
  await tick(GIVE_UP_MILLISECONDS / POLL_MILLISECONDS + 1);
  expect(screen.getByText("The portal did not come back")).toBeInTheDocument();
  const before = fetch.mock.calls.length;
  fireEvent.click(screen.getByRole("button", { name: "Try again" }));
  await tick(2);
  expect(fetch.mock.calls.length).toBeGreaterThan(before);
  expect(screen.getByText("The portal is restarting…")).toBeInTheDocument();
});
