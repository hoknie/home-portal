import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, expect, it, vi } from "vitest";

import { automationsKey, automationsSchema, catalogueKey, catalogueSchema, runsKey, runsSchema } from "@/entities/automation";
import { webhooksKey, webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { AutomationsScreen } from "./automations-screen";

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(""),
  usePathname: () => "/admin/automations/",
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

function renderWith(automations: unknown) {
  const client = testQueryClient();
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

it("lists each automation with its trigger in words, its script and its last run", () => {
  renderWith(apiSamples.automations);
  const table = screen.getAllByRole("table")[0];
  expect(within(table).getByRole("link", { name: "Restart Jellyfin when it goes down" }).getAttribute("href")).toMatch(
    /^\/admin\/automations\/edit\/?\?id=restart-media$/,
  );
  expect(within(table).getByText("Jellyfin: down, could not check")).toBeInTheDocument();
  expect(within(table).getByText("cron 0 3 * * *")).toBeInTheDocument();
  expect(within(table).getByText("Disabled")).toBeInTheDocument();
  expect(within(table).getByText("Has not run yet")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Add automation" }).getAttribute("href")).toMatch(/^\/admin\/automations\/new\/?$/);
});

it("a failed run in the journal shows its exit code and its error output", async () => {
  renderWith(apiSamples.automations);
  const journal = screen.getAllByRole("table")[1];
  const failed = within(journal).getAllByRole("row")[1];
  expect(within(failed).getByText("Failed")).toBeInTheDocument();
  await userEvent.click(within(failed).getByRole("button", { name: "Open" }));
  const sheet = await screen.findByRole("dialog");
  expect(within(sheet).getByText("Exit code")).toBeInTheDocument();
  expect(within(sheet).getByText("1")).toBeInTheDocument();
  expect(within(sheet).getByText("disk full")).toBeInTheDocument();
});

it("merged skips say how many times they happened", () => {
  renderWith(apiSamples.automations);
  expect(screen.getAllByText("9 times").length).toBeGreaterThan(0);
});

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
  expect(within(table()).getAllByRole("row")).toHaveLength(3);
  await userEvent.click(within(screen.getByRole("group", { name: "Tags:" })).getByRole("button", { name: "backup" }));
  expect(within(table()).getAllByRole("row")).toHaveLength(2);
  expect(within(table()).getByText("Nightly backup")).toBeInTheDocument();
});

it("the journal asks for the runs of a webhook and a text, and refreshes on demand", async () => {
  const fetch = vi.fn(async (path: string) =>
    path.startsWith("/api/webhooks") ? jsonResponse(apiSamples.webhooks) : jsonResponse(apiSamples.automationRuns),
  );
  vi.stubGlobal("fetch", fetch);
  renderWith(apiSamples.automations);
  screen.getByRole("combobox", { name: "Show runs of" }).focus();
  await userEvent.keyboard("{Enter}");
  await userEvent.click(await screen.findByRole("option", { name: "Deploy from CI" }));
  await userEvent.type(screen.getByLabelText("Search in values"), "main");
  await waitFor(() =>
    expect(fetch.mock.calls.map(([path]) => path)).toContain(
      "/api/automations/runs?webhook=7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d&text=main",
    ),
  );
  expect(screen.getByRole("switch", { name: "Auto-refresh" })).toBeChecked();
  expect(screen.getByText(/^Updated at \d{1,2}:\d{2}:\d{2}\s[AP]M$/)).toBeInTheDocument();
  const before = fetch.mock.calls.length;
  await userEvent.click(screen.getByRole("button", { name: "Refresh now" }));
  await waitFor(() => expect(fetch.mock.calls.length).toBeGreaterThan(before));
});
