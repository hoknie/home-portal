import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, expect, it, vi } from "vitest";

import { automationsKey, automationsSchema, catalogueKey, catalogueSchema, runsKey, runsSchema } from "@/entities/automation";
import { modulesKey, modulesSchema } from "@/entities/module";
import { sessionKey } from "@/entities/session";
import { webhooksKey, webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { AutomationsScreen } from "./automations-screen";

const navigation = vi.hoisted(() => ({ push: vi.fn() }));
const toast = vi.hoisted(() => ({ success: vi.fn(), error: vi.fn() }));
vi.mock("sonner", () => ({ toast }));

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(""),
  usePathname: () => "/admin/automations/",
  useRouter: () => ({ replace: vi.fn(), push: navigation.push }),
}));

function renderWith(automations: unknown, automationsOn = true, rights: Record<string, string[]> | null = null) {
  const client = testQueryClient();
  if (rights !== null) {
    client.setQueryData(sessionKey, { name: "anna", group: "family", admin: false, rights });
  }
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  modules.modules[2].enabled = automationsOn;
  client.setQueryDefaults(modulesKey, { staleTime: Infinity });
  client.setQueryData(modulesKey, { data: modules, revision: '"m"' });
  client.setQueryDefaults(catalogueKey, { staleTime: Infinity });
  client.setQueryDefaults(automationsKey, { staleTime: Infinity });
  client.setQueryDefaults(runsKey(), { staleTime: Infinity });
  client.setQueryData(automationsKey, { data: automationsSchema.parse(automations), revision: '"r"' });
  client.setQueryData(catalogueKey, catalogueSchema.parse(apiSamples.automationCatalogue));
  client.setQueryData(runsKey(), runsSchema.parse(apiSamples.automationRuns));
  client.setQueryDefaults(webhooksKey, { staleTime: Infinity });
  client.setQueryData(webhooksKey, { data: webhooksSchema.parse(apiSamples.webhooks), revision: '"r"' });
  return renderWithProviders(<AutomationsScreen />, client);
}

it("lists each automation with its trigger in words, its action and its last run, and no journal", () => {
  renderWith(apiSamples.automations);
  const table = screen.getAllByRole("table")[0];
  expect(within(table).getByRole("link", { name: "Restart Jellyfin when it goes down" }).getAttribute("href")).toMatch(
    /^\/admin\/automations\/edit\/?\?id=restart-media$/,
  );
  expect(within(table).getByText("Jellyfin: down, could not check")).toBeInTheDocument();
  expect(within(table).getByText("cron 0 3 * * *")).toBeInTheDocument();
  expect(within(table).getByText("Disabled")).toBeInTheDocument();
  expect(within(table).getByText("Running")).toBeInTheDocument();
  expect(within(table).getByText("for 12 s")).toBeInTheDocument();
  expect(within(table).getByRole("button", { name: "Stop" })).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Add automation" }).getAttribute("href")).toMatch(/^\/admin\/automations\/new\/?$/);
  expect(screen.getAllByRole("table")).toHaveLength(1);
});

const samples = runsSchema.parse(apiSamples.automationRuns).runs;

function serving(runs: Record<string, unknown>) {
  const fetch = vi.fn(async (path: string, init?: RequestInit) => {
    const id = /\/api\/automations\/runs\/(\d+)/.exec(path)?.[1];
    if (id && path.endsWith("/stop") && init?.method === "POST") {
      return jsonResponse(runs[id], { status: 202 });
    }
    return id ? jsonResponse(runs[id]) : jsonResponse(apiSamples.automationRuns);
  });
  vi.stubGlobal("fetch", fetch);
  return fetch;
}

it("invites the first automation when there are none", () => {
  renderWith({ automations: [] });
  expect(screen.getByText("No automations yet")).toBeInTheDocument();
});

beforeAll(() => {
  Element.prototype.hasPointerCapture = () => false;
  Element.prototype.scrollIntoView = () => {};
});

afterEach(() => {
  vi.unstubAllGlobals();
});

it("filters the automations by their shared tags", async () => {
  renderWith(apiSamples.automations);
  const table = () => screen.getAllByRole("table")[0];
  expect(within(table()).getAllByRole("row")).toHaveLength(4);
  await userEvent.click(within(screen.getByRole("group", { name: "Tags:" })).getByRole("button", { name: "backup" }));
  expect(within(table()).getAllByRole("row")).toHaveLength(2);
  expect(within(table()).getByText("Nightly backup")).toBeInTheDocument();
});

it("stopping from the table asks first, calls the endpoint and says so", async () => {
  const fetch = serving({ "5": { ...samples[0], outcome: { ...samples[0].outcome, reason: "stopping" } } });
  renderWith(apiSamples.automations);
  const table = screen.getAllByRole("table")[0];
  await userEvent.click(within(table).getByRole("button", { name: "Stop" }));
  expect(screen.getByText("Stop run 5 of “Nightly backup”?")).toBeInTheDocument();
  await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Stop" }));
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/automations/runs/5/stop", expect.objectContaining({ method: "POST" })));
});

it("automations while off: the list stays, the notice says so, and nothing can be run", () => {
  renderWith(apiSamples.automations, false);
  expect(screen.getAllByRole("status").some((element) => element.textContent?.includes("The Automations module is off"))).toBe(true);
  const table = screen.getAllByRole("table")[0];
  expect(within(table).getByRole("link", { name: "Restart Jellyfin when it goes down" })).toBeInTheDocument();
  const runButtons = within(table).getAllByRole("button", { name: "Run now" });
  expect(runButtons.length).toBeGreaterThan(0);
  for (const button of runButtons) {
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute("title", "The module is off");
  }
});

it("automations while on show no module notice", () => {
  renderWith(apiSamples.automations);
  expect(screen.queryByText(/The Automations module is off/)).not.toBeInTheDocument();
});

it("running without editing: the list and Run now are shown, adding, editing and deleting are not", () => {
  renderWith(apiSamples.automations, true, { automations: ["read", "execute"] });
  const table = screen.getAllByRole("table")[0];
  expect(within(table).getAllByRole("button", { name: /Run now/ }).length).toBeGreaterThan(0);
  expect(screen.queryByRole("link", { name: "Add automation" })).toBeNull();
  expect(within(table).queryByRole("link", { name: "Edit" })).toBeNull();
  expect(within(table).queryByRole("button", { name: /Delete/ })).toBeNull();
});

it("an admin sees adding, editing and deleting", () => {
  renderWith(apiSamples.automations);
  const table = screen.getAllByRole("table")[0];
  expect(screen.getByRole("link", { name: "Add automation" })).toBeInTheDocument();
  expect(within(table).getAllByRole("link", { name: "Edit" }).length).toBeGreaterThan(0);
  expect(within(table).getAllByRole("button", { name: /Delete/ }).length).toBeGreaterThan(0);
});

it("the action column names a script as text without the scripts page, and links a workflow", () => {
  renderWith(apiSamples.automations);
  const table = screen.getAllByRole("table")[0];
  const backup = within(table).getByText("backup/nightly.sh");
  expect(backup.closest("a")).toBeNull();
  expect(within(backup.closest("tr") as HTMLElement).getByText("Script")).toBeInTheDocument();
  const revive = within(table).getByText("revive");
  expect(within(revive.closest("tr") as HTMLElement).getByText("Workflow")).toBeInTheDocument();
});

it("the toast of a queued run leads to that run on the journal page", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(apiSamples.automationQueued, { status: 202 })));
  renderWith(apiSamples.automations);
  const table = screen.getAllByRole("table")[0];
  const row = within(table).getByRole("link", { name: "Restart Jellyfin when it goes down" }).closest("tr") as HTMLElement;
  await userEvent.click(within(row).getByRole("button", { name: "Run now" }));
  await userEvent.click(screen.getAllByRole("button", { name: "Run now" }).at(-1)!);
  await waitFor(() => expect(toast.success).toHaveBeenCalled());
  const options = toast.success.mock.calls[0][1] as { action: { onClick: () => void } };
  options.action.onClick();
  expect(navigation.push).toHaveBeenCalledWith(`/admin/runs/?run=${apiSamples.automationQueued.run_id}`);
});
