import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { automationsKey, automationsSchema, runsKey, runsSchema } from "@/entities/automation";
import { webhooksKey, webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { RunJournalScreen } from "./run-journal-screen";

let search = "";
const navigation = vi.hoisted(() => ({ replace: vi.fn() }));

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(search),
  usePathname: () => "/admin/runs/",
  useRouter: () => ({ replace: navigation.replace, push: vi.fn() }),
}));

afterEach(() => {
  search = "";
  navigation.replace.mockReset();
  vi.unstubAllGlobals();
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

function renderWith(automations: unknown) {
  const client = testQueryClient();
  client.setQueryDefaults(automationsKey, { staleTime: Infinity });
  client.setQueryDefaults(runsKey(), { staleTime: Infinity });
  client.setQueryData(automationsKey, { data: automationsSchema.parse(automations), revision: '"r"' });
  client.setQueryData(runsKey(), runsSchema.parse(apiSamples.automationRuns));
  client.setQueryDefaults(webhooksKey, { staleTime: Infinity });
  client.setQueryData(webhooksKey, { data: webhooksSchema.parse(apiSamples.webhooks), revision: '"r"' });
  return renderWithProviders(<RunJournalScreen />, client);
}

it("a failed run in the journal shows its exit code and its error output", async () => {
  serving({ "3": samples[2] });
  renderWith(apiSamples.automations);
  const journal = screen.getAllByRole("table")[0];
  const failed = within(journal).getAllByRole("row")[3];
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

it("an open running run follows its output until it finishes", async () => {
  const running = samples[0];
  const later = { ...running, outcome: { ...running.outcome, stdout: { tail: "copying\ndone\n", bytes: 13, truncated: false } } };
  const finished = { ...later, outcome: { ...later.outcome, result: "succeeded", exit_code: 0 } };
  const answers = [running, later, finished];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (path: string) => (path === "/api/automations/runs/5" ? jsonResponse(answers.shift() ?? finished) : jsonResponse(apiSamples.automationRuns))),
  );
  renderWith(apiSamples.automations);
  const journal = screen.getAllByRole("table")[0];
  await userEvent.click(within(within(journal).getAllByRole("row")[1]).getByRole("button", { name: "Open" }));
  const sheet = await screen.findByRole("dialog");
  expect(await within(sheet).findByText("Running")).toBeInTheDocument();
  expect(within(sheet).getByRole("button", { name: "Stop" })).toBeInTheDocument();
  expect(await within(sheet).findByText(/done/, {}, { timeout: 3000 })).toBeInTheDocument();
  expect(await within(sheet).findByText("Succeeded", {}, { timeout: 3000 })).toBeInTheDocument();
  expect(within(sheet).queryByRole("button", { name: "Stop" })).toBeNull();
});

it("the journal of one workflow asks for its runs", async () => {
  const fetch = vi.fn(async (path: string) =>
    path.startsWith("/api/workflows") ? jsonResponse(apiSamples.workflows) : path.startsWith("/api/webhooks") ? jsonResponse(apiSamples.webhooks) : jsonResponse(apiSamples.automationRuns),
  );
  vi.stubGlobal("fetch", fetch);
  renderWith(apiSamples.automations);
  screen.getByRole("combobox", { name: "Show runs of" }).focus();
  await userEvent.keyboard("{Enter}");
  await userEvent.click(await screen.findByRole("option", { name: "Revive a service" }));
  await waitFor(() => expect(fetch.mock.calls.map(([path]) => path)).toContain("/api/automations/runs?workflow=revive"));
});

it("an address naming a run opens its details, and closing it goes back to the journal's address", async () => {
  search = "run=3";
  serving({ "3": samples[2] });
  renderWith(apiSamples.automations);
  const sheet = await screen.findByRole("dialog");
  expect(await within(sheet).findByText("disk full")).toBeInTheDocument();
  await userEvent.keyboard("{Escape}");
  await waitFor(() => expect(navigation.replace).toHaveBeenCalledWith("/admin/runs/", { scroll: false }));
});

it("opening a run puts it in the address", async () => {
  serving({ "3": samples[2] });
  renderWith(apiSamples.automations);
  const journal = screen.getAllByRole("table")[0];
  await userEvent.click(within(within(journal).getAllByRole("row")[3]).getByRole("button", { name: "Open" }));
  expect(navigation.replace).toHaveBeenCalledWith("/admin/runs/?run=3", { scroll: false });
});

function paged() {
  const answer = (before: string | null) =>
    before === null
      ? { runs: apiSamples.automationRuns.runs, next_before: "100" }
      : before === "100"
        ? { runs: apiSamples.automationRuns.runs.slice(1, 3), next_before: "50" }
        : { runs: apiSamples.automationRuns.runs.slice(3, 4), next_before: null };
  const fetch = vi.fn(async (path: string) =>
    path.startsWith("/api/automations/runs") ? jsonResponse(answer(new URL(path, "http://portal").searchParams.get("before"))) : jsonResponse(apiSamples.webhooks),
  );
  vi.stubGlobal("fetch", fetch);
  return fetch;
}

it("older runs come a page at a time, the last page cannot go older, and an older page says it does not refresh", async () => {
  const fetch = paged();
  renderWithProviders(<RunJournalScreen />);
  await userEvent.click(await screen.findByRole("button", { name: "Older" }));
  await waitFor(() => expect(fetch.mock.calls.map(([path]) => path)).toContain("/api/automations/runs?before=100"));
  await userEvent.click(await screen.findByRole("button", { name: "Older" }));
  await waitFor(() => expect(screen.getByRole("button", { name: "Older" })).toBeDisabled());
  expect(screen.getByText(/Page 3: older runs do not refresh themselves/)).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Newer" }));
  expect(await screen.findByText(/Page 2/)).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Go to the newest" }));
  expect(await screen.findByText("Page 1")).toBeInTheDocument();
});

it("choosing a filter on an older page starts over from the newest runs", async () => {
  const fetch = paged();
  renderWithProviders(<RunJournalScreen />);
  await userEvent.click(await screen.findByRole("button", { name: "Older" }));
  await screen.findByText(/Page 2/);
  await userEvent.type(screen.getByLabelText("Search in values"), "main");
  await waitFor(() => expect(fetch.mock.calls.map(([path]) => path)).toContain("/api/automations/runs?text=main"));
  expect(await screen.findByText("Page 1")).toBeInTheDocument();
});
