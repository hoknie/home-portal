import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { modulesKey, modulesSchema } from "@/entities/module";
import { workflowsKey, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { WorkflowsScreen } from "./workflows-screen";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

beforeEach(() => {
  window.history.replaceState(null, "", "/admin/workflows/");
});

afterEach(() => {
  vi.unstubAllGlobals();
});

function render(workflowsOn: boolean) {
  const client = testQueryClient();
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  const withWorkflows = { ...modules, modules: modules.modules.map((module) => (module.name === "workflows" ? { ...module, enabled: workflowsOn } : module)) };
  client.setQueryDefaults(modulesKey, { staleTime: Infinity });
  client.setQueryData(modulesKey, { data: withWorkflows, revision: '"m"' });
  client.setQueryDefaults(workflowsKey, { staleTime: Infinity });
  client.setQueryData(workflowsKey, { data: workflowsSchema.parse(apiSamples.workflows), revision: '"r"' });
  return renderWithProviders(<WorkflowsScreen />, client);
}

it("with no workflows the page explains them and offers starter templates", () => {
  const client = testQueryClient();
  client.setQueryDefaults(workflowsKey, { staleTime: Infinity });
  client.setQueryData(workflowsKey, { data: { workflows: [] }, revision: '"r"' });
  renderWithProviders(<WorkflowsScreen />, client);
  expect(screen.getByText(/A workflow is a chain of steps/)).toBeInTheDocument();
  const retry = screen.getByRole("link", { name: /Retry a request until it answers/ });
  expect(retry.getAttribute("href")).toMatch(/^\/admin\/workflows\/new\/?\?template=retry$/);
});

it("add workflow offers the same gallery when workflows exist", async () => {
  render(true);
  await userEvent.click(screen.getByRole("button", { name: "Add workflow" }));
  const dialog = await screen.findByRole("dialog", { name: "Add a workflow" });
  expect(within(dialog).getByRole("link", { name: /Empty workflow/ }).getAttribute("href")).toMatch(/template=empty$/);
});

function rowOf(title: string) {
  return screen.getByRole("link", { name: title }).closest("tr")!;
}

it("a workflow in use names who starts it, and delete is disabled with the reason", () => {
  render(true);
  const row = rowOf("Revive a service");
  expect(within(row).getByText("NAS down")).toBeInTheDocument();
  const remove = within(row).getByRole("button", { name: "Delete" });
  expect(remove).toBeDisabled();
  expect(remove.parentElement).toHaveAttribute("title", "Used by NAS down; change them first");
  expect(within(rowOf("Note")).getByRole("button", { name: "Delete" })).toBeEnabled();
});

it("run now asks for the inputs, warns, and leads to the run on the workflow's page", async () => {
  const fetch = vi.fn(async (path: string) =>
    path.endsWith("/run") ? jsonResponse({ run_id: "13" }) : jsonResponse(workflowsSchema.parse(apiSamples.workflows).workflows[0].last_run),
  );
  vi.stubGlobal("fetch", fetch);
  render(true);
  await userEvent.click(within(rowOf("Revive a service")).getByRole("button", { name: "Run now" }));
  const dialog = await screen.findByRole("dialog");
  expect(within(dialog).getByText(/really run/)).toBeInTheDocument();
  await userEvent.type(within(dialog).getByLabelText("service"), "nas");
  await userEvent.click(within(dialog).getByRole("button", { name: "Run now" }));
  expect(fetch).toHaveBeenCalledWith("/api/workflows/revive/run", expect.objectContaining({ method: "POST", body: JSON.stringify({ inputs: { service: "nas", tries: 3 } }) }));
  await waitFor(() => expect(window.location.pathname).toBe("/admin/workflows/revive/history/13/"));
});

it("workflows while off: the list stays, the notice shows and run now is disabled", () => {
  render(false);
  expect(screen.getByRole("status")).toHaveTextContent("The Workflows module is off");
  expect(within(rowOf("Revive a service")).getByRole("button", { name: "Run now" })).toBeDisabled();
});

it("the runs of a workflow on its row: the history button and the last run lead to the workflow's page", async () => {
  render(true);
  const row = rowOf("Revive a service");
  const lastRun = workflowsSchema.parse(apiSamples.workflows).workflows[0].last_run!;
  expect(within(row).getByRole("link", { name: "Runs of Revive a service" })).toHaveAttribute("href", "/admin/workflows/revive/history/");
  expect(row.querySelector(`a[href="/admin/workflows/revive/history/${lastRun.id}/"]`)).not.toBeNull();
  await userEvent.click(within(row).getByRole("link", { name: "Runs of Revive a service" }));
  expect(window.location.pathname).toBe("/admin/workflows/revive/history/");
  expect(screen.queryByRole("dialog")).toBeNull();
});

it("opening a workflow from the list leads to its page, and Edit to its editor", async () => {
  render(true);
  const row = rowOf("Revive a service");
  expect(within(row).getByRole("link", { name: "Edit" })).toHaveAttribute("href", "/admin/workflows/revive/edit/");
  await userEvent.click(within(row).getByRole("link", { name: "Revive a service" }));
  expect(window.location.pathname).toBe("/admin/workflows/revive/");
});

it("run now builds its form from the input types and sends a list", async () => {
  const fetch = vi.fn(async (path: string) =>
    path.endsWith("/run") ? jsonResponse({ run_id: "14" }) : jsonResponse(workflowsSchema.parse(apiSamples.workflows).workflows[0].last_run),
  );
  vi.stubGlobal("fetch", fetch);
  const client = testQueryClient();
  const sample = workflowsSchema.parse(apiSamples.workflows);
  const typed = {
    ...sample.workflows[0],
    inputs: [
      { name: "hosts", type: "list" as const, default: ["nas"], description: "Hosts to check" },
      { name: "deep", type: "boolean" as const, default: null, description: null },
    ],
  };
  client.setQueryDefaults(workflowsKey, { staleTime: Infinity });
  client.setQueryData(workflowsKey, { data: { workflows: [typed] }, revision: '"r"' });
  renderWithProviders(<WorkflowsScreen />, client);
  await userEvent.click(within(rowOf("Revive a service")).getByRole("button", { name: "Run now" }));
  const dialog = await screen.findByRole("dialog");
  expect(within(dialog).getByText(/Hosts to check/)).toBeInTheDocument();
  expect(within(dialog).getByLabelText("Value 1")).toHaveValue("nas");
  await userEvent.click(within(dialog).getByRole("button", { name: "Add a value" }));
  await userEvent.type(within(dialog).getByLabelText("Value 2"), "router");
  await userEvent.click(within(dialog).getByRole("switch", { name: "deep" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Run now" }));
  expect(fetch).toHaveBeenCalledWith(
    "/api/workflows/revive/run",
    expect.objectContaining({ method: "POST", body: JSON.stringify({ inputs: { hosts: ["nas", "router"], deep: true } }) }),
  );
});

it("the list page's breadcrumbs lead from home to workflows", () => {
  render(true);
  expect(screen.getByRole("navigation", { name: "Breadcrumbs" })).toHaveTextContent("HomeWorkflows");
});
