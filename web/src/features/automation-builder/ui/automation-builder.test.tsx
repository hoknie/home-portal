import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { automationsSchema, catalogueSchema, scriptsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { AutomationBuilder } from "./automation-builder";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
const scripts = scriptsSchema.parse(apiSamples.automationScripts);
const saved = automationsSchema.parse(apiSamples.automations).automations[0];

afterEach(() => {
  vi.unstubAllGlobals();
});

function network() {
  const fetch = vi.fn(async (path: string) =>
    path.startsWith("/api/automations/schedule") ? jsonResponse(apiSamples.automationSchedule) : jsonResponse(saved, { status: 201 }),
  );
  vi.stubGlobal("fetch", fetch);
  return fetch;
}

function open(onSaved = vi.fn()) {
  renderWithProviders(
    <AutomationBuilder automation={null} revision='"r1"' taken={[]} catalogue={catalogue} scripts={scripts} onSaved={onSaved} onConflict={vi.fn()} />,
  );
  return onSaved;
}

function group(name: string) {
  const found = screen.getAllByRole("group", { name }).find((element) => element.tagName !== "OPTGROUP");
  expect(found).toBeDefined();
  return found as HTMLElement;
}

async function sentBody(fetch: ReturnType<typeof network>) {
  await waitFor(() => expect(fetch.mock.calls.some(([path]) => path === "/api/automations")).toBe(true));
  const [, init] = fetch.mock.calls.find(([path]) => path === "/api/automations") as unknown as [string, RequestInit];
  return JSON.parse(String(init.body));
}

it("builds a restart rule with a field pressed into the second argument", async () => {
  const fetch = network();
  const onSaved = open();
  await userEvent.type(screen.getByLabelText("Title"), "Restart Jellyfin");
  expect(screen.getByLabelText("Id")).toHaveValue("restart-jellyfin");
  await userEvent.click(within(group("Services")).getByLabelText("Jellyfin"));
  await userEvent.click(within(group("To state")).getByLabelText("Down"));
  await userEvent.selectOptions(screen.getByLabelText(/^Script/), "restart.sh");
  await userEvent.click(screen.getByRole("button", { name: "Add argument" }));
  await userEvent.type(screen.getByLabelText("Argument 1"), "--service");
  await userEvent.click(screen.getByRole("button", { name: "Add argument" }));
  await userEvent.click(screen.getByLabelText("Argument 2"));
  await userEvent.click(screen.getByRole("button", { name: "service.id" }));
  expect(screen.getByLabelText("Argument 2")).toHaveValue("{{service.id}}");
  expect(screen.getByTestId("command-line")).toHaveTextContent("restart.sh '--service' 'jellyfin'");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const body = await sentBody(fetch);
  expect(body.when).toEqual({ event: "service.status-changed", services: ["jellyfin"], to: ["down"] });
  expect(body.run).toEqual({ script: "restart.sh", args: ["--service", "{{service.id}}"], timeout_seconds: 60 });
  await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
});

it("a daily preset writes the expression and lists the next five times", async () => {
  network();
  open();
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "schedule");
  await userEvent.selectOptions(screen.getByLabelText("Preset"), "daily");
  expect(screen.getByLabelText(/^Cron expression/)).toHaveValue("0 3 * * *");
  const list = await screen.findByRole("list", { name: "Next runs" }, { timeout: 3000 });
  expect(within(list).getAllByRole("listitem")).toHaveLength(5);
  expect(screen.getByText("Next runs (Europe/Berlin)")).toBeInTheDocument();
});

it("changing the event drops the filters it does not have", async () => {
  const fetch = network();
  open();
  await userEvent.type(screen.getByLabelText("Title"), "Audit");
  await userEvent.click(within(group("To state")).getByLabelText("Down"));
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "user.signed-in");
  expect(screen.queryByRole("group", { name: "To state" })).not.toBeInTheDocument();
  expect(group("Users")).toBeInTheDocument();
  expect(group("Environments")).toBeInTheDocument();
  await userEvent.selectOptions(screen.getByLabelText(/^Script/), "restart.sh");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const body = await sentBody(fetch);
  expect(body.when).toEqual({ event: "user.signed-in" });
});

it("a field of another event is marked and keeps the form from saving", async () => {
  network();
  open();
  await userEvent.click(screen.getByRole("button", { name: "service.id" }));
  expect(screen.getByLabelText("Argument 1")).toHaveValue("{{service.id}}");
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "portal.started");
  expect(await screen.findByText("The argument names service.id, which this event does not have")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
});

it("a script that cannot run is offered but not choosable, with its reason", () => {
  network();
  open();
  const option = screen.getByRole("option", { name: /open\.sh/ });
  expect(option).toBeDisabled();
  expect(option).toHaveTextContent("open.sh can be written by group or others");
});

it("a title that would give an id the interface reserves gets another id", async () => {
  network();
  open();
  await userEvent.type(screen.getByLabelText("Title"), "Schedule");
  expect(screen.getByLabelText("Id")).toHaveValue("schedule-2");
});

it("a webhook event offers only event webhooks, their variables and the body as fields", async () => {
  network();
  open();
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "webhook.received");
  expect(screen.getByRole("button", { name: "webhook.title" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "webhook.body" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "webhook.camera" })).toBeInTheDocument();
  expect(within(group("Webhooks")).queryByLabelText(/Deploy from CI/)).not.toBeInTheDocument();
  await userEvent.click(within(group("Webhooks")).getByLabelText("Motion at the door"));
  expect(screen.getByRole("button", { name: "webhook.camera" })).toBeInTheDocument();
});

it("a chosen webhook that never publishes the event is marked", () => {
  network();
  const automation = automationsSchema.parse(apiSamples.automations).automations[0];
  renderWithProviders(
    <AutomationBuilder
      automation={{ ...automation, when: { ...automation.when, event: "webhook.received", services: [], to: [], webhooks: ["7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d"] } }}
      revision='"r1"'
      taken={[]}
      catalogue={catalogue}
      scripts={scripts}
      onSaved={vi.fn()}
      onConflict={vi.fn()}
    />,
  );
  expect(within(group("Webhooks")).getByLabelText("Deploy from CI (never publishes this event)")).toBeChecked();
});

it("a manual automation has no filters and no fields of its own", async () => {
  network();
  open();
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "manual");
  expect(screen.getByText("This event has no filters.")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "service.id" })).not.toBeInTheDocument();
});

it("an argument completes event fields after {{ with their samples", async () => {
  network();
  open();
  await userEvent.click(screen.getByRole("button", { name: "Add argument" }));
  await userEvent.click(screen.getByLabelText("Argument 1"));
  await userEvent.keyboard("{{{{ser");
  const options = screen.getAllByRole("option").map((option) => option.textContent ?? "");
  expect(options.some((text) => text.includes("service.id") && text.includes("nas"))).toBe(true);
  expect(options.some((text) => text.includes("service.name"))).toBe(true);
  await userEvent.keyboard("{Enter}");
  expect(screen.getByLabelText("Argument 1")).toHaveValue("{{service.id}}");
});

it("a background refresh does not change the revision it sends, and a conflict can be overwritten", async () => {
  const fetch = vi.fn(async (path: string, init?: RequestInit) =>
    init?.method === "PUT" && fetch.mock.calls.filter(([, sent]) => sent?.method === "PUT").length === 1 ? new Response("stale", { status: 409 }) : path.startsWith("/api/automations/schedule") ? jsonResponse(apiSamples.automationSchedule) : jsonResponse(saved),
  );
  vi.stubGlobal("fetch", fetch);
  const builder = (revision: string) => <AutomationBuilder automation={saved} revision={revision} taken={[]} catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />;
  const view = renderWithProviders(builder('"r1"'));
  view.rerender(builder('"r2"'));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await userEvent.click(await screen.findByRole("button", { name: "Overwrite" }));
  await waitFor(() => expect(fetch.mock.calls.filter(([, sent]) => sent?.method === "PUT")).toHaveLength(2));
  const matches = fetch.mock.calls.filter(([, sent]) => sent?.method === "PUT").map(([, sent]) => (sent?.headers as Record<string, string>)["If-Match"]);
  expect(matches).toEqual(['"r1"', '"r2"']);
});
