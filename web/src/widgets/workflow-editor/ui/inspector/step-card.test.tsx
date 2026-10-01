import { fireEvent, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { runsSchema, traceSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { TRACE_ENTRY_BLANKS, jsonResponse } from "@/shared/lib/testing";

import { node, openAt, stubCanvasDom, withSteps } from "../testing-support";

vi.mock("next/navigation", () => ({ usePathname: () => null, useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const fresh = { last_run: null, active_run: null };

beforeEach(() => {
  stubCanvasDom();
  window.localStorage.setItem("home-portal.workflow-editor.legend-dismissed", "1");
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ runs: [] })));
});

afterEach(() => {
  vi.unstubAllGlobals();
  window.localStorage.clear();
});

async function cardOf(steps: Parameters<typeof withSteps>[0], name: string, scripts?: Parameters<typeof openAt>[2]) {
  openAt(withSteps(steps, fresh), "/admin/workflows/draft/", scripts);
  fireEvent.click(await node(name));
  return screen.getByRole("complementary", { name });
}

function noInputs(column: HTMLElement) {
  expect(column.querySelector("input, textarea, select, [role=switch], [role=combobox]")).toBeNull();
}

it("reading an http step without editing it: method, URL chips, a default switch in words and what it produces", async () => {
  const column = await cardOf([{ id: "ping", kind: "http", url: "http://{{inputs.host}}/ping" }], "ping");
  expect(within(column).getByText("HTTP request")).toBeInTheDocument();
  const url = within(column).getByText("URL").nextElementSibling as HTMLElement;
  expect(url).toHaveTextContent("http://{{inputs.host}}/ping");
  expect(within(url).getByText("{{inputs.host}}").className).toContain("bg-primary");
  expect(within(column).getByText("Method").nextElementSibling).toHaveTextContent("GET(default)");
  expect(within(column).getByText("Fail on status 400 and above").nextElementSibling).toHaveTextContent("yes(default)");
  expect(within(column).getByText("steps.ping.status", { exact: false })).toBeInTheDocument();
  noInputs(column);
  await userEvent.click(within(column).getByRole("button", { name: "Close" }));
  expect(screen.queryByRole("complementary")).toBeNull();
});

it("an if reads its condition in words, a set its rows, and a script its named arguments", async () => {
  const check = await cardOf([{ id: "check", kind: "if", condition: { left: "{{inputs.host}}", op: "==", right: "nas" } }], "check");
  expect(check).toHaveTextContent("equals");
  noInputs(check);
});

it("a set step with a list shows its rows", async () => {
  const column = await cardOf([{ id: "names", kind: "set", variable: "names", list: ["nas", "{{inputs.host}}"] }], "names");
  const rows = within(column).getAllByRole("listitem").map((item) => item.textContent);
  expect(rows).toEqual(expect.arrayContaining(["nas", "{{inputs.host}}"]));
  noInputs(column);
});

it("a script step shows its arguments by their declared names", async () => {
  const declared = [{ name: "service", option: false, required: true, type: "text" as const, choices: [], default: null, description: "" }];
  const column = await cardOf([{ id: "run", kind: "script", script: "restart.sh", args: ["{{inputs.host}}"] }], "run", [{ path: "restart.sh", runnable: true, problem: null, arguments: declared }]);
  const row = within(column).getByText("service").closest("li");
  expect(row).toHaveTextContent("service{{inputs.host}}");
  expect(within(column).getByText("Continue the run when the script fails").nextElementSibling).toHaveTextContent("no(default)");
  noInputs(column);
});

it("choosing a node on the runs replaces the list with the card, and Close brings the list back", async () => {
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/history/");
  expect(await screen.findByRole("complementary", { name: "History" })).toBeInTheDocument();
  fireEvent.click(await node("ping"));
  const card = screen.getByRole("complementary", { name: "ping" });
  await userEvent.click(within(card).getByRole("button", { name: "Close" }));
  expect(screen.getByRole("complementary", { name: "History" })).toBeInTheDocument();
});

it("on a run's address the step data has a closed Settings that opens to the card", async () => {
  const [base] = runsSchema.parse(apiSamples.automationRuns).runs;
  const trace = traceSchema.parse({ entries: [{ ...TRACE_ENTRY_BLANKS, path: "steps[0]", step: "ping", label: "ping", kind: "http", iteration: null, outcome: "succeeded", started_at: "2026-09-29T10:00:00Z", duration_milliseconds: 5, detail: "GET http://nas.lan → 200", output: null }], dropped: 0 });
  const shown = { ...base, id: "9", workflow: "draft", trace };
  vi.stubGlobal("fetch", vi.fn(async (path: string) => (path === "/api/automations/runs/9" ? jsonResponse(shown) : jsonResponse({ runs: [shown] }))));
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/history/9/");
  fireEvent.click(await node("ping"));
  const details = await screen.findByRole("region", { name: "Step details" });
  const settings = await within(details).findByText("Settings");
  const folded = settings.closest("details")!;
  expect(folded).not.toHaveAttribute("open");
  const log = within(details).getByText("GET http://nas.lan → 200");
  expect(log.compareDocumentPosition(folded) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(log.closest("li")).not.toHaveAttribute("style");
  await userEvent.click(settings);
  expect(folded).toHaveAttribute("open");
  expect(within(folded).getByText("URL").nextElementSibling).toHaveTextContent("http://nas.lan");
  expect(within(folded).queryByText("Produces")).toBeNull();
});

it("History opens the runs even while a step's card is open", async () => {
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/");
  fireEvent.click(await node("ping"));
  expect(screen.getByRole("complementary", { name: "ping" })).toBeInTheDocument();
  await userEvent.click(within(screen.getByRole("toolbar", { name: "Workflow actions" })).getByRole("link", { name: "History" }));
  expect(await screen.findByRole("complementary", { name: "History" })).toBeInTheDocument();
  expect(screen.queryByRole("complementary", { name: "ping" })).toBeNull();
});
