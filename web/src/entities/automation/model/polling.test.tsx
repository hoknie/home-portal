import { waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { useRun } from "./queries";

afterEach(() => {
  vi.unstubAllGlobals();
});

function RunProbe({ id }: { id: string }) {
  useRun(id);
  return null;
}

it("a run that answers 404 is asked for once and then no more", async () => {
  const fetch = vi.fn(async () => jsonResponse({ error: "no such run" }, { status: 404 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RunProbe id="999" />);
  await waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
  await new Promise((resolve) => setTimeout(resolve, 2_500));
  expect(fetch).toHaveBeenCalledTimes(1);
});
