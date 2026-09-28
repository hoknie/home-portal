import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { type Run, runsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { jsonResponse } from "@/shared/lib/testing";

import { node, openEditor, stubCanvasDom, withSteps } from "../testing-support";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const [base] = runsSchema.parse(apiSamples.automationRuns).runs;

beforeEach(() => {
  stubCanvasDom();
  window.localStorage.setItem("home-portal.workflow-editor.legend-dismissed", "1");
});

afterEach(() => {
  vi.unstubAllGlobals();
  window.localStorage.clear();
});

type Entry = NonNullable<Run["trace"]>["entries"][number];

function entry(path: string, step: string, kind: string, outcome: Entry["outcome"], detail = ""): Entry {
  return { path, step, label: step, kind, iteration: null, outcome, started_at: "2026-09-28T03:00:00Z", duration_milliseconds: 15, detail, output: null, shape: null, values: [], log: [], values_dropped: 0, log_dropped: 0, item: null, level: null };
}

function run(result: Run["outcome"]["result"], entries: Entry[]): Run {
  return { ...base, id: "7", workflow: "draft", trace: { entries, dropped: 0 }, outcome: { ...base.outcome, result } };
}

it("watching a run on the canvas colours the nodes that ran and emphasises the branch taken", async () => {
  const first = run("running", [entry("steps[0]", "ping", "http", "succeeded"), entry("steps[1]", "check", "if", "running")]);
  const second = run("succeeded", [
    entry("steps[0]", "ping", "http", "succeeded"),
    entry("steps[1]", "check", "if", "succeeded", "else"),
    entry("steps[1].else[0]", "later", "wait", "succeeded"),
  ]);
  let polls = 0;
  const fetch = vi.fn(async (path: string) => {
    if (path.endsWith("/run")) {
      return jsonResponse({ run_id: "7" });
    }
    polls += 1;
    return jsonResponse(polls === 1 ? first : second);
  });
  vi.stubGlobal("fetch", fetch);
  openEditor(
    withSteps([
      { id: "ping", kind: "http", url: "http://nas.lan" },
      {
        id: "check",
        kind: "if",
        condition: { left: "{{steps.ping.status}}", op: "==", right: "200" },
        then: [{ id: "done", kind: "stop", outcome: "succeeded" }],
        else: [{ id: "later", kind: "wait", seconds: 1 }],
      },
    ]),
  );
  await userEvent.click(screen.getByRole("button", { name: "Run" }));
  const dialog = await screen.findByRole("dialog");
  expect(within(dialog).getByText(/really run/)).toBeInTheDocument();
  await userEvent.click(within(dialog).getByRole("button", { name: "Run now" }));
  expect(fetch).toHaveBeenCalledWith("/api/workflows/draft/run", expect.objectContaining({ method: "POST" }));
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "succeeded"));
  expect(await node("check")).toHaveAttribute("data-outcome", "running");
  expect(within(screen.getByRole("complementary", { name: "Run" })).getByRole("button", { name: /Stop/ })).toBeInTheDocument();
  await waitFor(async () => expect(await node("later")).toHaveAttribute("data-outcome", "succeeded"), { timeout: 4000 });
  expect(await node("check")).toHaveAttribute("data-outcome", "succeeded");
  expect((await node("done")).hasAttribute("data-outcome")).toBe(false);
  expect(await node("done")).toHaveAttribute("data-dimmed", "true");
  expect(await node("ping")).toHaveAttribute("data-order", "1");
  expect(await node("later")).toHaveAttribute("data-order", "3");
  expect(screen.getByText("otherwise").className).toContain("border-primary");
  expect(screen.getByText("then").className).not.toContain("border-primary");
  await userEvent.click(screen.getByRole("button", { name: "Hide run" }));
  expect((await node("ping")).hasAttribute("data-outcome")).toBe(false);
});

it("replaying an older run from the runs panel draws its path with the failure in red and what follows dimmed", async () => {
  const failed = run("failed", [entry("steps[0]", "ping", "http", "failed")]);
  const fetch = vi.fn(async (path: string) => (path.startsWith("/api/automations/runs?workflow=draft") ? jsonResponse({ runs: [{ ...failed, id: "3" }] }) : jsonResponse({ ...failed, id: "3" })));
  vi.stubGlobal("fetch", fetch);
  openEditor(
    withSteps([
      { id: "ping", kind: "http", url: "http://nas.lan" },
      { id: "tell", kind: "notify", text: "x" },
    ]),
  );
  await userEvent.click(screen.getByRole("button", { name: "History" }));
  const list = await screen.findByRole("list", { name: "Runs of this workflow" });
  await userEvent.click(within(list).getAllByRole("button")[0]);
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "failed"));
  expect(await node("ping")).toHaveAttribute("data-order", "1");
  expect(await node("tell")).toHaveAttribute("data-dimmed", "true");
  const strip = screen.getByRole("list", { name: "Steps of the run" });
  await userEvent.click(within(strip).getByRole("button", { name: /ping/ }));
  expect(screen.getByRole("complementary")).toBeInTheDocument();
});

it("a run shown on the canvas feeds the key suggestions, from its answer's shape when the body was cut", async () => {
  const answered = { ...entry("steps[0]", "ping", "http", "succeeded"), output: '{"disks":[{"na', shape: '{"disks":[{"name":"sda"}],"state":"up"}' };
  const shown = run("succeeded", [answered]);
  const fetch = vi.fn(async (path: string) => (path.startsWith("/api/automations/runs?workflow=draft") ? jsonResponse({ runs: [{ ...shown, id: "4" }] }) : jsonResponse({ ...shown, id: "4" })));
  vi.stubGlobal("fetch", fetch);
  openEditor(
    withSteps([
      { id: "ping", kind: "http", url: "http://nas.lan" },
      { id: "tell", kind: "notify", text: "" },
    ]),
  );
  await userEvent.click(screen.getByRole("button", { name: "History" }));
  const list = await screen.findByRole("list", { name: "Runs of this workflow" });
  await userEvent.click(within(list).getAllByRole("button")[0]);
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "succeeded"));
  await userEvent.click(screen.getByRole("radio", { name: "Edit" }));
  fireEvent.click(await node("tell"));
  const text = within(screen.getByRole("complementary")).getByRole("combobox", { name: "Text" });
  await userEvent.click(text);
  await userEvent.keyboard("{{{{steps.ping.json.");
  const offered = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(offered.some((option) => option.startsWith("steps.ping.json.disks"))).toBe(true);
  expect(offered.some((option) => option.startsWith("steps.ping.json.state"))).toBe(true);
});

it("no editing while a run is shown: no palette slots, no node menus, no delete, and the node shows its log", async () => {
  const logged = { ...entry("steps[0]", "ping", "http", "failed", "GET http://nas.lan → 500"), values: [{ template: "http://{{inputs.host}}", value: '"http://nas.lan"' }], log: ["GET http://nas.lan → 500 in 12 ms"] };
  const shown = run("failed", [logged]);
  vi.stubGlobal(
    "fetch",
    vi.fn(async (path: string) => (path.endsWith("/run") ? jsonResponse({ run_id: "7" }) : jsonResponse(shown))),
  );
  openEditor(
    withSteps([
      { id: "ping", kind: "http", url: "http://nas.lan" },
      { id: "tell", kind: "notify", text: "x" },
    ]),
  );
  expect(screen.getAllByRole("button", { name: "Add a step here" }).length).toBeGreaterThan(0);
  await userEvent.click(screen.getByRole("button", { name: "Run" }));
  await userEvent.click(within(await screen.findByRole("dialog")).getByRole("button", { name: "Run now" }));
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "failed"));
  expect(screen.getByRole("radio", { name: "View" })).toHaveAttribute("aria-checked", "true");
  expect(screen.queryAllByRole("button", { name: "Add a step here" })).toHaveLength(0);
  expect(screen.queryByRole("button", { name: "Actions for ping" })).toBeNull();
  expect(screen.getByRole("button", { name: /^Save( \(|$)/ })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Undo" })).toBeDisabled();
  fireEvent.click(await node("tell"));
  fireEvent.keyDown(await node("tell"), { key: "Delete" });
  expect(await node("tell")).toBeInTheDocument();
  fireEvent.click(await node("ping"));
  const panel = screen.getByRole("complementary");
  expect(within(panel).queryByRole("combobox", { name: "URL" })).toBeNull();
  expect(within(panel).getByRole("region", { name: "Values" })).toHaveTextContent('"http://nas.lan"');
  expect(within(panel).getByRole("region", { name: "Log" })).toHaveTextContent("in 12 ms");
});

it("back to editing keeps unsaved changes, and a run of an older version carries a note", async () => {
  const older = { ...run("succeeded", [entry("steps[0]", "ping", "http", "succeeded")]), id: "3", steps_version: "aaaaaaaaaaaa" };
  vi.stubGlobal(
    "fetch",
    vi.fn(async (path: string) => (path.startsWith("/api/automations/runs?workflow=draft") ? jsonResponse({ runs: [older] }) : jsonResponse(older))),
  );
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], { steps_version: "bbbbbbbbbbbb" }));
  fireEvent.click(await node("ping"));
  const url = within(screen.getByRole("complementary")).getByRole("combobox", { name: "URL" });
  await userEvent.clear(url);
  await userEvent.type(url, "http://router.lan");
  await userEvent.click(screen.getByRole("button", { name: "History" }));
  await userEvent.click(within(await screen.findByRole("list", { name: "Runs of this workflow" })).getAllByRole("button")[0]);
  expect(await screen.findByText(/ran an earlier version/)).toBeInTheDocument();
  await userEvent.click(screen.getByRole("radio", { name: "Edit" }));
  expect(screen.queryByText(/ran an earlier version/)).toBeNull();
  fireEvent.click(await node("ping"));
  expect(within(screen.getByRole("complementary")).getByRole("combobox", { name: "URL" })).toHaveValue("http://router.lan");
  expect(screen.getByRole("button", { name: /^Save( \(|$)/ })).toBeEnabled();
});

it("the run panel lists the steps in order and shows one pass of a step at a time", async () => {
  const pass = (iteration: number, outcome: Entry["outcome"], log: string) => ({ ...entry("steps[0].body[0]", "check", "status", outcome), iteration, item: `"s${iteration}"`, log: [log] });
  const looped = { ...run("failed", [entry("steps[0]", "each", "loop", "failed"), pass(0, "succeeded", "s0 → up"), pass(1, "failed", "s1 → no service")]), id: "5" };
  vi.stubGlobal(
    "fetch",
    vi.fn(async (path: string) => (path.startsWith("/api/automations/runs?workflow=draft") ? jsonResponse({ runs: [looped] }) : jsonResponse(looped))),
  );
  openEditor(withSteps([{ id: "each", kind: "loop", for_each: "{{inputs.hosts}}", body: [{ id: "check", kind: "status", service: "{{loop.item}}" }] }]));
  await userEvent.click(screen.getByRole("button", { name: "History" }));
  await userEvent.click(within(await screen.findByRole("list", { name: "Runs of this workflow" })).getAllByRole("button")[0]);
  const panel = await screen.findByRole("complementary", { name: "Run" });
  const steps = within(panel).getByRole("list", { name: "Steps of the run" });
  expect(within(steps).getByText('Pass 1: "s0"')).toBeInTheDocument();
  expect(within(steps).getByText('Pass 2: "s1"')).toBeInTheDocument();
  fireEvent.click(await node("check"));
  const details = within(panel).getByRole("region", { name: "Step details" });
  expect(within(details).getByText("s1 → no service")).toBeInTheDocument();
  expect(within(details).queryByText("s0 → up")).toBeNull();
  await userEvent.click(within(within(details).getByRole("group", { name: "Passes of this step" })).getByRole("button", { name: "1" }));
  expect(within(details).getByText("s0 → up")).toBeInTheDocument();
  expect(within(details).queryByText("s1 → no service")).toBeNull();
});
