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

function open() {
  renderWithProviders(
    <AutomationBuilder automation={null} revision='"r1"' taken={[]} catalogue={catalogue} scripts={scripts} onSaved={vi.fn()} onConflict={vi.fn()} />,
  );
}

async function sentRun(fetch: ReturnType<typeof network>) {
  await waitFor(() => expect(fetch.mock.calls.some(([path]) => path === "/api/automations")).toBe(true));
  const [, init] = fetch.mock.calls.find(([path]) => path === "/api/automations") as unknown as [string, RequestInit];
  return JSON.parse(String(init.body)).run;
}

function group(name: string) {
  const found = screen.getAllByRole("group", { name }).find((element) => element.tagName !== "OPTGROUP");
  expect(found).toBeDefined();
  return found as HTMLElement;
}

it("a script's declared arguments become fields that write the args in their order", async () => {
  const fetch = network();
  open();
  await userEvent.type(screen.getByLabelText("Title"), "Restart Jellyfin");
  await userEvent.click(within(group("Services")).getByLabelText("Jellyfin"));
  await userEvent.selectOptions(screen.getByLabelText(/^Script/), "media/restart.sh");
  expect(screen.getByText("Restart a service's container")).toBeInTheDocument();
  expect(screen.getByLabelText("--retries")).toHaveAttribute("placeholder", "3");
  await userEvent.click(screen.getByLabelText("service"));
  await userEvent.click(screen.getByRole("button", { name: "service.id" }));
  expect(screen.getByLabelText("service")).toHaveValue("{{service.id}}");
  await userEvent.click(screen.getByRole("switch", { name: "--force" }));
  expect(screen.getByTestId("command-line")).toHaveTextContent("media/restart.sh 'jellyfin' '--force'");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect((await sentRun(fetch)).args).toEqual(["{{service.id}}", "--force"]);
});

it("hand-written args that do not follow the declared form stay a list, with a note", async () => {
  const fetch = network();
  open();
  await userEvent.type(screen.getByLabelText("Title"), "Force");
  await userEvent.click(screen.getByRole("button", { name: "Add argument" }));
  await userEvent.type(screen.getByLabelText("Argument 1"), "--force");
  await userEvent.click(screen.getByRole("button", { name: "Add argument" }));
  await userEvent.type(screen.getByLabelText("Argument 2"), "nas");
  await userEvent.selectOptions(screen.getByLabelText(/^Script/), "media/restart.sh");
  expect(screen.getByText("These arguments do not follow what the script declares, so they are kept as a list.")).toBeInTheDocument();
  expect(screen.getByLabelText("Argument 1")).toHaveValue("--force");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect((await sentRun(fetch)).args).toEqual(["--force", "nas"]);
});

it("fields can be switched to the plain list and back", async () => {
  network();
  open();
  await userEvent.selectOptions(screen.getByLabelText(/^Script/), "media/restart.sh");
  await userEvent.type(screen.getByLabelText("service"), "nas");
  await userEvent.click(screen.getByRole("button", { name: "Edit as a list" }));
  expect(screen.getByLabelText("Argument 1")).toHaveValue("nas");
  await userEvent.click(screen.getByRole("button", { name: "Edit as fields" }));
  expect(screen.getByLabelText("service")).toHaveValue("nas");
});
