import { screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { automationsKey, automationsSchema, catalogueKey, catalogueSchema, scriptsKey, scriptsSchema } from "@/entities/automation";
import { notificationsKey, notificationsSchema } from "@/entities/notification";
import { secretNamesKey, workflowCatalogueKey, workflowCatalogueSchema, workflowsKey, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { stubCanvasDom } from "./testing-support";
import { WorkflowEditorScreen } from "./workflow-editor-screen";

let search = "id=revive";

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(search),
  usePathname: () => "/admin/workflows/edit/",
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
}));

beforeEach(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

function renderScreen(mode: "new" | "edit") {
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
  return renderWithProviders(<WorkflowEditorScreen mode={mode} />, client);
}

it("a workflow's editor names the workflow in its breadcrumbs", () => {
  search = "id=revive";
  renderScreen("edit");
  const trail = screen.getByRole("navigation", { name: "Breadcrumbs" });
  expect(within(trail).getByRole("link", { name: "Home" })).toHaveAttribute("href", "/");
  expect(within(trail).getByRole("link", { name: "Workflows" }).getAttribute("href")).toMatch(/^\/admin\/workflows\/?$/);
  expect(within(trail).getByText("Revive a service")).toHaveAttribute("aria-current", "page");
});

it("a new workflow and an unknown id have their own last item", () => {
  search = "";
  const first = renderScreen("new");
  expect(screen.getByRole("navigation", { name: "Breadcrumbs" })).toHaveTextContent("HomeWorkflowsNew workflow");
  first.unmount();
  search = "id=nope";
  renderScreen("edit");
  expect(screen.getByRole("navigation", { name: "Breadcrumbs" })).toHaveTextContent("HomeWorkflowsnope");
});
