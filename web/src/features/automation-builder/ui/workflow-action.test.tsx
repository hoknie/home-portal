import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { automationsSchema, catalogueSchema, scriptsSchema } from "@/entities/automation";
import { modulesKey, modulesSchema } from "@/entities/module";
import { workflowsKey, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { AutomationBuilder } from "./automation-builder";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
const scripts = scriptsSchema.parse(apiSamples.automationScripts);
const saved = automationsSchema.parse(apiSamples.automations).automations[0];

afterEach(() => {
  vi.unstubAllGlobals();
});

function open(workflowsOn: boolean) {
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
    <AutomationBuilder automation={null} revision='"r1"' taken={[]} catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />,
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
