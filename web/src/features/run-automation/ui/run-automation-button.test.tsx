import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { automationsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { RunAutomationButton } from "./run-automation-button";

const toast = vi.hoisted(() => ({ success: vi.fn(), error: vi.fn() }));
vi.mock("sonner", () => ({ toast }));

const [restart, backup] = automationsSchema.parse(apiSamples.automations).automations;

afterEach(() => {
  vi.unstubAllGlobals();
  toast.success.mockReset();
  toast.error.mockReset();
});

it("warns that the script really runs and queues it only after confirmation", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.automationQueued, { status: 202 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RunAutomationButton automation={restart} labelled />);
  await userEvent.click(screen.getByRole("button", { name: "Run now" }));
  expect(screen.getByText(/really runs on the portal's host/)).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.click(screen.getAllByRole("button", { name: "Run now" }).at(-1)!);
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/automations/restart-media/run", expect.objectContaining({ method: "POST" })));
  expect(toast.success).toHaveBeenCalledWith("“Restart Jellyfin when it goes down” is queued to run");
});

it("the toast opens the queued run", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(apiSamples.automationQueued, { status: 202 })));
  const onQueued = vi.fn();
  renderWithProviders(<RunAutomationButton automation={restart} labelled onQueued={onQueued} />);
  await userEvent.click(screen.getByRole("button", { name: "Run now" }));
  await userEvent.click(screen.getAllByRole("button", { name: "Run now" }).at(-1)!);
  await waitFor(() => expect(toast.success).toHaveBeenCalled());
  const [message, options] = toast.success.mock.calls[0] as [string, { action: { label: string; onClick: () => void } }];
  expect(message).toBe("“Restart Jellyfin when it goes down” is queued to run");
  expect(options.action.label).toBe("Open");
  options.action.onClick();
  expect(onQueued).toHaveBeenCalledWith("42");
});

it("a second run too soon says how long to wait", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("too soon", { status: 429, headers: { "Retry-After": "4" } })));
  renderWithProviders(<RunAutomationButton automation={restart} labelled />);
  await userEvent.click(screen.getByRole("button", { name: "Run now" }));
  await userEvent.click(screen.getAllByRole("button", { name: "Run now" }).at(-1)!);
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith("Wait 4 s before the next run"));
});

it("a disabled automation cannot be started", () => {
  renderWithProviders(<RunAutomationButton automation={backup} />);
  expect(screen.getByRole("button", { name: "Run now" })).toBeDisabled();
});
