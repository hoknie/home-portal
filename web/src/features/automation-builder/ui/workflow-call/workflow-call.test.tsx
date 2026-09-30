import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Automation, automationsSchema, catalogueSchema, scriptsSchema } from "@/entities/automation";
import { modulesKey, modulesSchema } from "@/entities/module";
import { workflowsKey, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { AutomationBuilder } from "../automation-builder";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
const scripts = scriptsSchema.parse(apiSamples.automationScripts);
const saved = automationsSchema.parse(apiSamples.automations).automations[0];

afterEach(() => {
  vi.unstubAllGlobals();
});

function open(workflowsOn: boolean, automation: Automation | null = null) {
  const client = testQueryClient();
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  client.setQueryDefaults(modulesKey, { staleTime: Infinity });
  client.setQueryData(modulesKey, {
    data: { ...modules, modules: modules.modules.map((module) => (module.name === "workflows" ? { ...module, enabled: workflowsOn } : module)) },
    revision: '"m"',
  });
  client.setQueryDefaults(workflowsKey, { staleTime: Infinity });
  client.setQueryData(workflowsKey, { data: workflowsSchema.parse(apiSamples.workflows), revision: '"w"' });
  renderWithProviders(
    <AutomationBuilder automation={automation} revision='"r1"' taken={[]} catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />,
    client,
  );
}

it("running a workflow from an automation writes the workflow and its inputs and no run", async () => {
  const fetch = vi.fn(async () => jsonResponse(saved, { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  open(true);
  await userEvent.type(screen.getByLabelText("Title"), "NAS down");
  await userEvent.click(screen.getByRole("radio", { name: "Workflow" }));
  await userEvent.selectOptions(screen.getByLabelText(/^Workflow/), "revive");
  await userEvent.click(screen.getByLabelText("Input service"));
  await userEvent.click(screen.getByRole("button", { name: "service.id" }));
  expect(screen.getByLabelText("Input service")).toHaveValue("{{service.id}}");
  expect(screen.queryByTestId("command-line")).toBeNull();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/automations", expect.objectContaining({ method: "POST" })));
  const body = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(body.workflow).toBe("revive");
  expect(body.inputs).toEqual({ service: "{{service.id}}" });
  expect(body.run).toBeUndefined();
});

it("the workflow action is not offered while the module is off", () => {
  open(false);
  expect(screen.queryByRole("radio", { name: "Workflow" })).toBeNull();
  expect(screen.getByLabelText(/^Script/)).toBeInTheDocument();
});

it("a webhook automation that runs a workflow is saved again after an edit", async () => {
  const fetch = vi.fn(async () => jsonResponse(saved));
  vi.stubGlobal("fetch", fetch);
  const loaded = automationsSchema.parse(apiSamples.automations).automations.find((automation) => automation.workflow !== null) as Automation;
  open(true, { ...loaded, run: { script: "", args: [], timeout_seconds: 0 } });
  await userEvent.type(screen.getByLabelText("Title"), " now");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalledWith(`/api/automations/${loaded.id}`, expect.objectContaining({ method: "PUT" })));
  const body = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(body.when.event).toBe("webhook.received");
  expect(body.workflow).toBe("revive");
  expect(body.inputs).toEqual({ service: "{{webhook.camera}}" });
  expect(body.run).toBeUndefined();
});

it("an error the form cannot place is listed above Save", async () => {
  const fetch = vi.fn(async () =>
    jsonResponse({ error: "invalid", errors: [{ field: "workflows[0].steps[1].url", message: "must be an address" }] }, { status: 422 }),
  );
  vi.stubGlobal("fetch", fetch);
  open(true);
  await userEvent.type(screen.getByLabelText("Title"), "NAS down");
  await userEvent.click(screen.getByRole("radio", { name: "Workflow" }));
  await userEvent.selectOptions(screen.getByLabelText(/^Workflow/), "revive");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("workflows[0].steps[1].url: must be an address")).toBeInTheDocument();
  expect(screen.getByLabelText("Title")).toHaveValue("NAS down");
});

function openWithHosts() {
  const client = testQueryClient();
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  client.setQueryDefaults(modulesKey, { staleTime: Infinity });
  client.setQueryData(modulesKey, { data: { ...modules, modules: modules.modules.map((module) => (module.name === "workflows" ? { ...module, enabled: true } : module)) }, revision: '"m"' });
  const workflows = workflowsSchema.parse(apiSamples.workflows);
  const hosts = { ...workflows.workflows[1], id: "check-hosts", title: "Check hosts", inputs: [{ name: "hosts", type: "list" as const, default: ["nas"], description: "Hosts to ping" }] };
  client.setQueryDefaults(workflowsKey, { staleTime: Infinity });
  client.setQueryData(workflowsKey, { data: { workflows: [...workflows.workflows, hosts] }, revision: '"w"' });
  renderWithProviders(
    <AutomationBuilder automation={null} revision='"r1"' taken={[]} catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />,
    client,
  );
}

async function chooseHosts() {
  await userEvent.type(screen.getByLabelText("Title"), "Hosts");
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "webhook.received");
  await userEvent.click(screen.getByRole("radio", { name: "Workflow" }));
  await userEvent.selectOptions(screen.getByLabelText(/^Workflow/), "check-hosts");
}

async function sentInputs(fetch: ReturnType<typeof vi.fn>) {
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  return JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
}

it("a list input typed by hand is saved as a list", async () => {
  const fetch = vi.fn(async () => jsonResponse(saved, { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  openWithHosts();
  await chooseHosts();
  expect(screen.getByText(/Hosts to ping · Type: List · Default: \["nas"\]/)).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Add a value" }));
  await userEvent.type(screen.getByLabelText("Value 1"), "nas");
  await userEvent.click(screen.getByRole("button", { name: "Add a value" }));
  await userEvent.type(screen.getByLabelText("Value 2"), "router");
  expect((await sentInputs(fetch)).inputs).toEqual({ hosts: ["nas", "router"] });
});

it("a list input from the webhook body is saved as its template", async () => {
  const fetch = vi.fn(async () => jsonResponse(saved, { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  openWithHosts();
  await chooseHosts();
  await userEvent.click(screen.getByLabelText("Write hosts as a template"));
  await userEvent.click(screen.getByRole("button", { name: "webhook.body" }));
  expect(screen.getByLabelText("Input hosts")).toHaveValue("{{webhook.body}}");
  expect((await sentInputs(fetch)).inputs).toEqual({ hosts: "{{webhook.body}}" });
});
