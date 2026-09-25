import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { catalogueSchema, scriptsSchema } from "@/entities/automation";
import { webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { WebhookForm } from "./webhook-form";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
const scripts = scriptsSchema.parse(apiSamples.automationScripts);

afterEach(() => {
  vi.unstubAllGlobals();
});

it("creates a webhook with a token and shows the token once with a curl call", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.webhookCreated, { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  const onSaved = vi.fn();
  renderWithProviders(<WebhookForm webhook={null} revision='"r1"' catalogue={catalogue} scripts={scripts} onSaved={onSaved} onConflict={vi.fn()} />);
  await userEvent.type(screen.getByLabelText("Title"), "Deploy");
  await userEvent.type(screen.getByLabelText(/Required\ variables/), "branch{Enter}");
  await userEvent.selectOptions(screen.getByLabelText(/^Script/), "restart.sh");
  await userEvent.click(screen.getByRole("button", { name: "webhook.branch" }));
  expect(screen.getByTestId("command-line")).toHaveTextContent("restart.sh 'branch'");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/webhooks");
  expect(JSON.parse(String(init.body))).toMatchObject({
    title: "Deploy",
    variables: ["branch"],
    action: "script",
    with_token: true,
    run: { script: "restart.sh", args: ["{{webhook.branch}}"] },
  });
  const dialog = await screen.findByRole("dialog");
  expect(within(dialog).getByText("f".repeat(64))).toBeInTheDocument();
  expect(within(dialog).getByText(/curl -X POST -H 'Authorization: Bearer f{64}'.*\/webhook\/0b9e8c2a/)).toBeInTheDocument();
  expect(onSaved).not.toHaveBeenCalled();
  await userEvent.click(within(dialog).getByRole("button", { name: "I have saved the token" }));
  expect(onSaved).toHaveBeenCalledOnce();
});

it("an event webhook needs no script, and an existing one offers its token actions", () => {
  const [, motion] = webhooksSchema.parse(apiSamples.webhooks).webhooks;
  renderWithProviders(<WebhookForm webhook={motion} revision='"r1"' catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />);
  expect(screen.getByLabelText(/Publish\ an\ event/)).toBeChecked();
  expect(screen.queryByLabelText(/^Script/)).not.toBeInTheDocument();
  expect(screen.getByText("Open to anyone")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Create a token" })).toBeInTheDocument();
});
