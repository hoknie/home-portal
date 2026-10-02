import { waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { useWidgetData } from "./queries";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

function WidgetProbe() {
  useWidgetData("box");
  return null;
}

it("a widget whose first fetch failed asks again after thirty seconds", async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  const fetch = vi.fn(async () => jsonResponse({ error: "down" }, { status: 502 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<WidgetProbe />);
  await waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
  await vi.advanceTimersByTimeAsync(31_000);
  await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
});

it("a widget still refreshing asks again after five seconds", async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  const fetch = vi.fn(async () => jsonResponse({ refreshing: true }, { status: 202 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<WidgetProbe />);
  await waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
  await vi.advanceTimersByTimeAsync(5_500);
  await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
});
