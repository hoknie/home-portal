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

it("a webhook event offers the variables of the chosen webhook as fields", async () => {
  network();
  open();
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "webhook.received");
  expect(screen.getByRole("button", { name: "webhook.title" })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "webhook.camera" })).not.toBeInTheDocument();
  await userEvent.click(within(group("Webhooks")).getByLabelText("Deploy from CI"));
  expect(screen.getByRole("button", { name: "webhook.branch" })).toBeInTheDocument();
  await userEvent.click(within(group("Webhooks")).getByLabelText("Motion at the door"));
  expect(screen.queryByRole("button", { name: "webhook.branch" })).not.toBeInTheDocument();
});

it("a manual automation has no filters and no fields of its own", async () => {
  network();
  open();
  await userEvent.selectOptions(screen.getByLabelText(/^Event/), "manual");
  expect(screen.getByText("This event has no filters.")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "service.id" })).not.toBeInTheDocument();
});
