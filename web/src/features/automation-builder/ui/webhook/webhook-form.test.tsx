import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { catalogueSchema, scriptsSchema } from "@/entities/automation";
import { webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { modulesKey, modulesSchema } from "@/entities/module";
import { workflowsKey, workflowsSchema } from "@/entities/workflow";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { WebhookForm } from "./webhook-form";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
const scripts = scriptsSchema.parse(apiSamples.automationScripts);

afterEach(() => {
  vi.unstubAllGlobals();
});

it("creates a webhook with a token and shows the token once with a curl call", async () => {
  const fetch = vi.fn<(path: string) => Promise<Response>>(async () => jsonResponse(apiSamples.webhookCreated, { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  const onSaved = vi.fn();
  renderWithProviders(<WebhookForm webhook={null} revision='"r1"' catalogue={catalogue} scripts={scripts} onSaved={onSaved} onConflict={vi.fn()} />);
  await userEvent.type(screen.getByLabelText("Title"), "Deploy");
  await userEvent.type(screen.getByLabelText(/Required\ variables/), "branch{Enter}");
  await userEvent.selectOptions(screen.getByLabelText(/^Script/), "restart.sh");
  await userEvent.click(screen.getByRole("button", { name: "webhook.branch" }));
  expect(screen.getByTestId("command-line")).toHaveTextContent("restart.sh 'branch'");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch.mock.calls.some(([path]) => path === "/api/webhooks")).toBe(true));
  const [, init] = fetch.mock.calls.find(([path]) => path === "/api/webhooks") as unknown as [string, RequestInit];
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

function withWorkflows() {
  const client = testQueryClient();
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  client.setQueryDefaults(modulesKey, { staleTime: Infinity });
  client.setQueryData(modulesKey, { data: { ...modules, modules: modules.modules.map((module) => (module.name === "workflows" ? { ...module, enabled: true } : module)) }, revision: '"m"' });
  client.setQueryDefaults(workflowsKey, { staleTime: Infinity });
  client.setQueryData(workflowsKey, { data: workflowsSchema.parse(apiSamples.workflows), revision: '"w"' });
  return client;
}

function sentBody(fetch: ReturnType<typeof vi.fn>, path: string) {
  const [, init] = fetch.mock.calls.find(([called]) => called === path) as unknown as [string, RequestInit];
  return JSON.parse(String(init.body));
}

it("a webhook that runs a workflow writes the workflow and its inputs and no run", async () => {
  const fetch = vi.fn<(path: string) => Promise<Response>>(async () => jsonResponse(apiSamples.webhookCreated, { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<WebhookForm webhook={null} revision='"r1"' catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />, withWorkflows());
  await userEvent.type(screen.getByLabelText("Title"), "Revive");
  await userEvent.type(screen.getByLabelText(/Required\ variables/), "service{Enter}");
  await userEvent.click(screen.getByLabelText(/Run\ a\ workflow/));
  await userEvent.selectOptions(screen.getByLabelText(/^Workflow/), "revive");
  await userEvent.click(screen.getByLabelText("Input service"));
  await userEvent.click(screen.getByRole("button", { name: "webhook.service" }));
  expect(screen.getByRole("button", { name: "webhook.body" })).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch.mock.calls.some(([path]) => path === "/api/webhooks")).toBe(true));
  const body = sentBody(fetch, "/api/webhooks");
  expect(body).toMatchObject({ action: "script", run: null, workflow: "revive", inputs: { service: "{{webhook.service}}" } });
});

it("editing a webhook that runs a workflow keeps the workflow", async () => {
  const fetch = vi.fn<(path: string) => Promise<Response>>(async () => jsonResponse(webhooksSchema.parse(apiSamples.webhooks).webhooks[2]));
  vi.stubGlobal("fetch", fetch);
  const release = webhooksSchema.parse(apiSamples.webhooks).webhooks[2];
  renderWithProviders(<WebhookForm webhook={release} revision='"r1"' catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />, withWorkflows());
  expect(screen.getByLabelText(/Run\ a\ workflow/)).toBeChecked();
  expect(screen.getByLabelText("Input service")).toHaveValue("{{webhook.service}}");
  await userEvent.type(screen.getByLabelText("Title"), " now");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const path = `/api/webhooks/${release.id}`;
  await waitFor(() => expect(fetch.mock.calls.some(([called]) => called === path)).toBe(true));
  expect(sentBody(fetch, path)).toMatchObject({ title: "Release from GitHub now", action: "script", run: null, workflow: "revive", inputs: { service: "{{webhook.service}}" } });
});
