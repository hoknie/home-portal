import { screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { automationsKey, automationsSchema, catalogueKey, catalogueSchema, scriptsKey, scriptsSchema } from "@/entities/automation";
import { notificationsKey, notificationsSchema } from "@/entities/notification";
import { secretNamesKey, workflowCatalogueKey, workflowCatalogueSchema, workflowsKey, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { stubCanvasDom } from "./testing-support";
import { WorkflowEditorScreen } from "./workflow-editor-screen";
import { WorkflowPage } from "./workflow-page";

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/workflows/",
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
}));

beforeEach(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

function primed() {
  const client = testQueryClient();
  for (const key of [workflowsKey, workflowCatalogueKey, scriptsKey, automationsKey, catalogueKey, secretNamesKey, notificationsKey]) {
    client.setQueryDefaults(key, { staleTime: Infinity });
  }
  client.setQueryData(workflowsKey, { data: workflowsSchema.parse(apiSamples.workflows), revision: '"r"' });
  client.setQueryData(workflowCatalogueKey, workflowCatalogueSchema.parse(apiSamples.workflowCatalogue));
  client.setQueryData(scriptsKey, scriptsSchema.parse(apiSamples.automationScripts));
  client.setQueryData(automationsKey, { data: automationsSchema.parse(apiSamples.automations), revision: '"a"' });
  client.setQueryData(catalogueKey, catalogueSchema.parse(apiSamples.automationCatalogue));
  client.setQueryData(secretNamesKey, []);
  client.setQueryData(notificationsKey, { data: notificationsSchema.parse(apiSamples.notifications), revision: '"n"' });
  return client;
}

function trail() {
  return screen.getByRole("navigation", { name: "Breadcrumbs" });
}

function trailText() {
  return (trail().textContent ?? "").replace("…", "");
}

it("a workflow's page names the workflow as the current page", () => {
  renderWithProviders(<WorkflowPage id="revive" view="view" run={null} onRunShown={vi.fn()} />, primed());
  expect(within(trail()).getByRole("link", { name: "Home" })).toHaveAttribute("href", "/");
  expect(within(trail()).getByRole("link", { name: "Workflows" }).getAttribute("href")).toMatch(/^\/admin\/workflows\/?$/);
  expect(within(trail()).getByText("Revive a service")).toHaveAttribute("aria-current", "page");
});

it("a workflow's editor ends in Edit, with the title linking to the workflow's page", () => {
  renderWithProviders(<WorkflowEditorScreen mode="edit" id="revive" template={null} lastShownRun={null} />, primed());
  expect(trailText()).toBe("HomeWorkflowsRevive a serviceEdit");
  expect(within(trail()).getByRole("link", { name: "Revive a service" })).toHaveAttribute("href", "/admin/workflows/revive/");
});

it("a workflow's run ends in History and the run, with History linking to the runs", () => {
  renderWithProviders(<WorkflowPage id="revive" view="run" run="42" onRunShown={vi.fn()} />, primed());
  expect(trailText()).toBe("HomeWorkflowsRevive a serviceHistoryRun 42");
  expect(within(trail()).getByRole("link", { name: "History" })).toHaveAttribute("href", "/admin/workflows/revive/history/");
});

it("a new workflow and an unknown id have their own last item, and the unknown one says so", () => {
  const first = renderWithProviders(<WorkflowEditorScreen mode="new" id={null} template={null} lastShownRun={null} />, primed());
  expect(trail()).toHaveTextContent("HomeWorkflowsNew workflow");
  first.unmount();
  renderWithProviders(<WorkflowPage id="nope" view="history" run={null} onRunShown={vi.fn()} />, primed());
  expect(trailText()).toBe("HomeWorkflowsnopeHistory");
  expect(screen.getByRole("link", { name: "Back to workflows" })).toHaveAttribute("href", "/admin/workflows/");
});
