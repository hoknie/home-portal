import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { RunWebhookDialog } from "./run-webhook-dialog";

const webhook = { ...webhooksSchema.parse(apiSamples.webhooks).webhooks[0], variables: ["branch"] };

afterEach(() => {
  vi.unstubAllGlobals();
});

it("running with a variable posts it and shows the run id", async () => {
  const fetch = vi.fn(async () => jsonResponse({ accepted: true, run_id: "42" }, { status: 202 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RunWebhookDialog webhook={webhook} />);
  await userEvent.click(screen.getByRole("button", { name: "Run now" }));
  const dialog = await screen.findByRole("dialog");
  const run = within(dialog).getByRole("button", { name: "Run" });
  expect(run).toBeDisabled();
  await userEvent.type(within(dialog).getByLabelText("branch"), "main");
  await userEvent.click(run);
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe(`/api/webhooks/${webhook.id}/run`);
  expect(JSON.parse(String(init.body))).toEqual({ variables: { branch: "main" } });
  expect(await within(dialog).findByRole("status")).toHaveTextContent("Run 42 was queued");
});

it("a refusal is shown in the dialog", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ errors: [{ field: "branch", message: "is required" }] }, { status: 422 })));
  renderWithProviders(<RunWebhookDialog webhook={webhook} />);
  await userEvent.click(screen.getByRole("button", { name: "Run now" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.type(within(dialog).getByLabelText("branch"), " x");
  await userEvent.click(within(dialog).getByRole("button", { name: "Run" }));
  expect(await within(dialog).findByText("is required")).toBeInTheDocument();
});
