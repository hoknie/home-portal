import { screen, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { runsSchema } from "../model/schema";
import { RunDetails } from "./run-details";

const [running, stopped] = runsSchema.parse(apiSamples.automationRuns).runs;

afterEach(() => {
  vi.unstubAllGlobals();
});

it("fetches the run it opens and offers the given actions", async () => {
  const fetch = vi.fn(async () => jsonResponse(running));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RunDetails runId="5" onClose={() => {}} actions={() => <button type="button">act</button>} />);
  const sheet = await screen.findByRole("dialog");
  expect(await within(sheet).findByText("Running")).toBeInTheDocument();
  expect(within(sheet).getByRole("button", { name: "act" })).toBeInTheDocument();
  expect(fetch).toHaveBeenCalledWith("/api/automations/runs/5", expect.anything());
});

it("says who stopped a run in the reader's language", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(stopped)));
  renderWithProviders(<RunDetails runId="4" onClose={() => {}} />);
  expect(await screen.findByText("Stopped by admin")).toBeInTheDocument();
});
