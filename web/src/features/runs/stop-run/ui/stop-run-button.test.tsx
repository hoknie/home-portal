import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { runsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { StopRunButton } from "./stop-run-button";

const toast = vi.hoisted(() => ({ success: vi.fn(), error: vi.fn() }));
vi.mock("sonner", () => ({ toast }));

const [running, stopped] = runsSchema.parse(apiSamples.automationRuns).runs;

afterEach(() => {
  vi.unstubAllGlobals();
  toast.success.mockReset();
  toast.error.mockReset();
});

it("stops a running run only after confirmation", async () => {
  const fetch = vi.fn(async () => jsonResponse({ ...running, outcome: { ...running.outcome, reason: "stopping" } }, { status: 202 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<StopRunButton run={running} title="Nightly backup" labelled />);
  await userEvent.click(screen.getByRole("button", { name: "Stop" }));
  expect(screen.getByText("Stop run 5 of “Nightly backup”?")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.click(screen.getAllByRole("button", { name: "Stop" }).at(-1)!);
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/automations/runs/5/stop", expect.objectContaining({ method: "POST" })));
  expect(toast.success).toHaveBeenCalledWith("Stopping run 5");
});

it("a run that finished meanwhile says so", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("the run has already finished", { status: 409 })));
  renderWithProviders(<StopRunButton run={running} labelled />);
  await userEvent.click(screen.getByRole("button", { name: "Stop" }));
  await userEvent.click(screen.getAllByRole("button", { name: "Stop" }).at(-1)!);
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith("The run has already finished"));
});

it("a finished run offers no stop, and a stopping one cannot be stopped twice", () => {
  const { container } = renderWithProviders(<StopRunButton run={stopped} />);
  expect(container).toBeEmptyDOMElement();
  renderWithProviders(<StopRunButton run={{ ...running, outcome: { ...running.outcome, reason: "stopping" } }} />);
  expect(screen.getByRole("button", { name: "Stop" })).toBeDisabled();
});
