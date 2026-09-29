import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { type Run, runsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { jsonResponse } from "@/shared/lib/testing";

import { node, openAt, stubCanvasDom, withSteps } from "../testing-support";

vi.mock("next/navigation", () => ({ usePathname: () => null, useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const [base] = runsSchema.parse(apiSamples.automationRuns).runs;

beforeEach(() => {
  stubCanvasDom();
  window.localStorage.setItem("home-portal.workflow-editor.legend-dismissed", "1");
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  window.localStorage.clear();
});

type Entry = NonNullable<Run["trace"]>["entries"][number];

function entry(path: string, step: string, kind: string, outcome: Entry["outcome"], detail = ""): Entry {
  return { path, step, label: step, kind, iteration: null, outcome, started_at: "2026-09-28T03:00:00Z", duration_milliseconds: 15, detail, output: null, shape: null, stdout: null, stderr: null, command: null, budget_reached: false, values: [], log: [], values_dropped: 0, log_dropped: 0, item: null, level: null, wait_seconds: null };
}

function run(result: Run["outcome"]["result"], entries: Entry[]): Run {
  return { ...base, id: "7", workflow: "draft", trace: { entries, dropped: 0 }, outcome: { ...base.outcome, result } };
}

const fresh = { last_run: null, active_run: null };

function serving(runs: Run[]) {
  return vi.fn(async (path: string) => {
    if (path.startsWith("/api/automations/runs?workflow=draft")) {
      return jsonResponse({ runs });
    }
    const found = runs.find((candidate) => path === `/api/automations/runs/${candidate.id}`);
    return found ? jsonResponse(found) : jsonResponse({ error: "not found" }, { status: 404 });
  });
}

function toolbar() {
  return screen.getByRole("toolbar", { name: "Workflow actions" });
}

it("opening an existing workflow shows it read-only across the whole width, with no run and no column beside it", async () => {
  const newest = { ...run("succeeded", [entry("steps[0]", "ping", "http", "succeeded")]), id: "12" };
  vi.stubGlobal("fetch", serving([newest]));
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], { last_run: newest, active_run: null }), "/admin/workflows/draft/");
  await node("ping");
  expect(within(toolbar()).getByRole("button", { name: "Run" })).toBeInTheDocument();
  expect(within(toolbar()).getByRole("link", { name: "History" })).toHaveAttribute("href", "/admin/workflows/draft/history/");
  expect(within(toolbar()).getByRole("link", { name: "Edit" })).toHaveAttribute("href", "/admin/workflows/draft/edit/");
  expect(within(toolbar()).getByRole("button", { name: "Delete" })).toBeInTheDocument();
  expect(screen.queryAllByRole("button", { name: "Add a step here" })).toHaveLength(0);
  expect(screen.queryByRole("complementary")).toBeNull();
  await new Promise((resolve) => setTimeout(resolve, 50));
  expect((await node("ping")).hasAttribute("data-outcome")).toBe(false);
});

it("closing the run and the runs returns to the workflow with no run and no column", async () => {
  const shown = { ...run("succeeded", [entry("steps[0]", "ping", "http", "succeeded")]), id: "42" };
  vi.stubGlobal("fetch", serving([shown]));
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/history/42/");
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "succeeded"));
  await userEvent.click(within(screen.getByRole("complementary", { name: "Run" })).getByRole("link", { name: "Close" }));
  expect(window.location.pathname).toBe("/admin/workflows/draft/");
  expect(screen.queryByRole("complementary")).toBeNull();
  expect((await node("ping")).hasAttribute("data-outcome")).toBe(false);
  await userEvent.click(within(toolbar()).getByRole("link", { name: "History" }));
  await userEvent.click(within(await screen.findByRole("complementary", { name: "History" })).getByRole("link", { name: "Close" }));
  expect(window.location.pathname).toBe("/admin/workflows/draft/");
  expect(screen.queryByRole("complementary")).toBeNull();
});

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
    if (path.startsWith("/api/automations/runs?workflow=draft")) {
      return jsonResponse({ runs: [second] });
    }
    if (path !== "/api/automations/runs/7") {
      return jsonResponse({ error: "not found" }, { status: 404 });
    }
    polls += 1;
    return jsonResponse(polls === 1 ? first : second);
  });
  vi.stubGlobal("fetch", fetch);
  openAt(
    withSteps(
      [
        { id: "ping", kind: "http", url: "http://nas.lan" },
        {
          id: "check",
          kind: "if",
          condition: { left: "{{steps.ping.status}}", op: "==", right: "200" },
          then: [{ id: "done", kind: "stop", outcome: "succeeded" }],
          else: [{ id: "later", kind: "wait", seconds: 1 }],
        },
      ],
      fresh,
    ),
    "/admin/workflows/draft/",
  );
  await node("ping");
  expect(screen.queryByRole("complementary")).toBeNull();
  await userEvent.click(within(toolbar()).getByRole("button", { name: "Run" }));
  const dialog = await screen.findByRole("dialog");
  expect(within(dialog).getByText(/really run/)).toBeInTheDocument();
  await userEvent.click(within(dialog).getByRole("button", { name: "Run now" }));
  expect(fetch).toHaveBeenCalledWith("/api/workflows/draft/run", expect.objectContaining({ method: "POST" }));
  await waitFor(() => expect(window.location.pathname).toBe("/admin/workflows/draft/history/7/"));
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
  await userEvent.click(screen.getByRole("link", { name: "All runs" }));
  expect(window.location.pathname).toBe("/admin/workflows/draft/history/");
  expect(await screen.findByRole("list", { name: "Runs of this workflow" })).toBeInTheDocument();
  expect((await node("ping")).hasAttribute("data-outcome")).toBe(false);
});

it("replaying an older run from History draws its path with the failure in red and what follows dimmed", async () => {
  const failed = { ...run("failed", [entry("steps[0]", "ping", "http", "failed")]), id: "3" };
  vi.stubGlobal("fetch", serving([failed]));
  openAt(
    withSteps(
      [
        { id: "ping", kind: "http", url: "http://nas.lan" },
        { id: "tell", kind: "notify", text: "x" },
      ],
      fresh,
    ),
    "/admin/workflows/draft/",
  );
  await userEvent.click(within(toolbar()).getByRole("link", { name: "History" }));
  expect(window.location.pathname).toBe("/admin/workflows/draft/history/");
  const list = await screen.findByRole("list", { name: "Runs of this workflow" });
  expect(screen.getByRole("complementary", { name: "History" })).toContainElement(list);
  await userEvent.click(within(list).getAllByRole("button")[0]);
  expect(window.location.pathname).toBe("/admin/workflows/draft/history/3/");
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "failed"));
  expect(await node("ping")).toHaveAttribute("data-order", "1");
  expect(await node("tell")).toHaveAttribute("data-dimmed", "true");
  const strip = screen.getByRole("list", { name: "Steps of the run" });
  await userEvent.click(within(strip).getByRole("button", { name: /ping/ }));
  expect(screen.getByRole("complementary", { name: "Run" })).toBeInTheDocument();
});

it("back and forward move through the workflow's addresses without a reload", async () => {
  const failed = { ...run("failed", [entry("steps[0]", "ping", "http", "failed")]), id: "3" };
  vi.stubGlobal("fetch", serving([failed]));
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/");
  await userEvent.click(within(toolbar()).getByRole("link", { name: "History" }));
  await userEvent.click(within(await screen.findByRole("list", { name: "Runs of this workflow" })).getAllByRole("button")[0]);
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "failed"));
  window.history.back();
  await waitFor(() => expect(window.location.pathname).toBe("/admin/workflows/draft/history/"));
  expect(await screen.findByRole("list", { name: "Runs of this workflow" })).toBeInTheDocument();
  window.history.back();
  await waitFor(() => expect(window.location.pathname).toBe("/admin/workflows/draft/"));
  await waitFor(() => expect(screen.queryByRole("complementary")).toBeNull());
  window.history.forward();
  expect(await screen.findByRole("list", { name: "Runs of this workflow" })).toBeInTheDocument();
});

it("a run's own address draws that run, links back to the runs, and an unknown run says so", async () => {
  const shown = { ...run("succeeded", [entry("steps[0]", "ping", "http", "succeeded")]), id: "42" };
  vi.stubGlobal("fetch", serving([shown]));
  const first = openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/history/42/");
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "succeeded"));
  expect(screen.getByRole("link", { name: "All runs" })).toHaveAttribute("href", "/admin/workflows/draft/history/");
  first.unmount();
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/history/99/");
  expect(await screen.findByText(/not one of this workflow's runs/)).toBeInTheDocument();
});

it("a run shown on the page feeds the editor's key suggestions, from its answer's shape when the body was cut", async () => {
  const answered = { ...entry("steps[0]", "ping", "http", "succeeded"), output: '{"disks":[{"na', shape: '{"disks":[{"name":"sda"}],"state":"up"}' };
  const shown = { ...run("succeeded", [answered]), id: "4" };
  vi.stubGlobal("fetch", serving([shown]));
  openAt(
    withSteps(
      [
        { id: "ping", kind: "http", url: "http://nas.lan" },
        { id: "tell", kind: "notify", text: "" },
      ],
      fresh,
    ),
    "/admin/workflows/draft/history/4/",
  );
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "succeeded"));
  await userEvent.click(within(toolbar()).getByRole("link", { name: "Edit" }));
  expect(window.location.pathname).toBe("/admin/workflows/draft/edit/");
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
  vi.stubGlobal("fetch", serving([run("failed", [logged])]));
  openAt(
    withSteps(
      [
        { id: "ping", kind: "http", url: "http://nas.lan" },
        { id: "tell", kind: "notify", text: "x" },
      ],
      fresh,
    ),
    "/admin/workflows/draft/history/7/",
  );
  await waitFor(async () => expect(await node("ping")).toHaveAttribute("data-outcome", "failed"));
  expect(screen.queryAllByRole("button", { name: "Add a step here" })).toHaveLength(0);
  expect(screen.queryByRole("button", { name: "Actions for ping" })).toBeNull();
  expect(screen.queryByRole("button", { name: /^Save/ })).toBeNull();
  expect(screen.queryByRole("button", { name: "Undo" })).toBeNull();
  fireEvent.click(await node("tell"));
  fireEvent.keyDown(await node("tell"), { key: "Delete" });
  expect(await node("tell")).toBeInTheDocument();
  fireEvent.click(await node("ping"));
  const panel = screen.getByRole("complementary", { name: "Run" });
  expect(within(panel).queryByRole("combobox", { name: "URL" })).toBeNull();
  expect(within(panel).getByRole("region", { name: "Values" })).toHaveTextContent('"http://nas.lan"');
  expect(within(panel).getByRole("region", { name: "Log" })).toHaveTextContent("in 12 ms");
});

it("back to editing shows the editable workflow without the run, and a run of an older version carries a note", async () => {
  const older = { ...run("succeeded", [entry("steps[0]", "ping", "http", "succeeded")]), id: "3", steps_version: "aaaaaaaaaaaa" };
  vi.stubGlobal("fetch", serving([older]));
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], { ...fresh, steps_version: "bbbbbbbbbbbb" }), "/admin/workflows/draft/history/3/");
  expect(await screen.findByText(/ran an earlier version/)).toBeInTheDocument();
  await userEvent.click(within(toolbar()).getByRole("link", { name: "Edit" }));
  expect(window.location.pathname).toBe("/admin/workflows/draft/edit/");
  expect(screen.queryByText(/ran an earlier version/)).toBeNull();
  expect((await node("ping")).hasAttribute("data-outcome")).toBe(false);
  fireEvent.click(await node("ping"));
  expect(within(screen.getByRole("complementary")).getByRole("combobox", { name: "URL" })).toBeInTheDocument();
});

it("cancelling an edit asks first and returns to the workflow's page as it was saved", async () => {
  vi.stubGlobal("fetch", serving([]));
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/edit/");
  fireEvent.click(await node("ping"));
  const url = within(screen.getByRole("complementary")).getByRole("combobox", { name: "URL" });
  await userEvent.clear(url);
  await userEvent.type(url, "http://router.lan");
  await userEvent.click(within(screen.getByRole("toolbar", { name: "Editor actions" })).getByRole("button", { name: "Cancel" }));
  expect(confirm).toHaveBeenCalled();
  expect(window.location.pathname).toBe("/admin/workflows/draft/");
  expect(await node("ping")).toHaveTextContent("GET http://nas.lan");
  expect(screen.getByRole("toolbar", { name: "Workflow actions" })).toBeInTheDocument();
});

it("saving leads to the workflow's page under its new id, not to the list", async () => {
  const renamed = { ...withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), id: "draft-2" };
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(renamed, { headers: { ETag: '"r2"' } })));
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], fresh), "/admin/workflows/draft/edit/");
  await node("ping");
  await userEvent.click(screen.getByRole("button", { name: /^Save$/ }));
  await waitFor(() => expect(window.location.pathname).toBe("/admin/workflows/draft-2/"));
});

it("the run panel lists the steps in order and shows one pass of a step at a time", async () => {
  const pass = (iteration: number, outcome: Entry["outcome"], log: string) => ({ ...entry("steps[0].body[0]", "check", "status", outcome), iteration, item: `"s${iteration}"`, log: [log] });
  const looped = { ...run("failed", [entry("steps[0]", "each", "loop", "failed"), pass(0, "succeeded", "s0 → up"), pass(1, "failed", "s1 → no service")]), id: "5" };
  vi.stubGlobal("fetch", serving([looped]));
  openAt(withSteps([{ id: "each", kind: "loop", for_each: "{{inputs.hosts}}", body: [{ id: "check", kind: "status", service: "{{loop.item}}" }] }], fresh), "/admin/workflows/draft/history/5/");
  const panel = await screen.findByRole("complementary", { name: "Run" });
  const steps = await within(panel).findByRole("list", { name: "Steps of the run" });
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

it("seeing why a script failed: the run panel names the last error line and Details opens the script's log", async () => {
  const failed = {
    ...entry("steps[0]", "restart", "script", "failed", "restart.sh exited 1: container not found"),
    stdout: { tail: "stopping\n", bytes: 9, truncated: false },
    stderr: { tail: "container not found\n", bytes: 20, truncated: false },
    command: ["restart.sh", "jellyfin"],
  };
  const shown = { ...run("failed", [failed]), id: "6" };
  vi.stubGlobal("fetch", serving([shown]));
  openAt(withSteps([{ id: "restart", kind: "script", script: "restart.sh", args: ["{{inputs.service}}"] }], fresh), "/admin/workflows/draft/history/6/");
  await waitFor(async () => expect(await node("restart")).toHaveAttribute("data-outcome", "failed"));
  fireEvent.click(await node("restart"));
  const details = within(screen.getByRole("complementary", { name: "Run" })).getByRole("region", { name: "Step details" });
  expect(within(details).getByText("restart.sh exited 1: container not found")).toBeInTheDocument();
  await userEvent.click(within(details).getByRole("button", { name: "Details" }));
  const dialog = await screen.findByRole("dialog", { name: "Log of restart" });
  expect(within(dialog).getByText("restart.sh 'jellyfin'")).toBeInTheDocument();
  expect(within(dialog).getByText("Exit code").nextElementSibling).toHaveTextContent("1");
  expect(within(dialog).getByText("Standard error").parentElement).toHaveTextContent("container not found");
});

it("a timer that runs counts down every second between answers and turns green when the wait ends", async () => {
  const waiting = run("running", [{ ...entry("steps[0]", "nap", "wait", "running"), duration_milliseconds: 0, wait_seconds: 60 }]);
  const done = run("succeeded", [{ ...entry("steps[0]", "nap", "wait", "succeeded", "60 s"), duration_milliseconds: 60_000, wait_seconds: 60 }]);
  const answers: ((response: Response) => void)[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (path: string) =>
      path === "/api/automations/runs/7"
        ? new Promise<Response>((resolve) => {
            answers.push(resolve);
          })
        : jsonResponse({ error: "not found" }, { status: 404 }),
    ),
  );
  openAt(withSteps([{ id: "nap", kind: "wait", seconds: 60 }], fresh), "/admin/workflows/draft/history/7/");
  await node("nap");
  await waitFor(() => expect(answers.length).toBeGreaterThan(0));
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "setInterval", "clearInterval", "Date"] });
  const tick = (milliseconds: number) =>
    act(async () => {
      await vi.advanceTimersByTimeAsync(milliseconds);
    });
  try {
    const badge = () => document.querySelector("[data-path='steps[0]'] [data-live='countdown']");
    answers[0](jsonResponse(waiting));
    for (let round = 0; round < 20 && badge() === null; round += 1) {
      await tick(10);
    }
    expect(badge()?.textContent).toBe("60 s");
    await tick(1_000);
    expect(badge()?.textContent).toBe("59 s");
    await tick(1_000);
    expect(badge()?.textContent).toBe("58 s");
    expect(badge()?.getAttribute("aria-label")).toBe("58 seconds left");
    answers.slice(1).forEach((answer) => answer(jsonResponse(done)));
    for (let round = 0; round < 50 && badge() !== null; round += 1) {
      await tick(100);
      answers.slice(1).forEach((answer) => answer(jsonResponse(done)));
    }
    expect(badge()).toBeNull();
    expect(document.querySelector("[data-path='steps[0]']")).toHaveAttribute("data-outcome", "succeeded");
  } finally {
    vi.useRealTimers();
  }
});

it("values on the nodes: a template shows what it gave as a chip naming the template, and Templates brings it back", async () => {
  const answered = { ...entry("steps[0]", "ping", "http", "succeeded"), values: [{ template: "http://{{inputs.host}}/ping", value: '"http://nas.lan/ping"' }] };
  vi.stubGlobal("fetch", serving([{ ...run("succeeded", [answered]), id: "8" }]));
  openAt(withSteps([{ id: "ping", kind: "http", url: "http://{{inputs.host}}/ping" }], fresh), "/admin/workflows/draft/history/8/");
  const ping = await node("ping");
  await waitFor(() => expect(ping).toHaveTextContent("GET http://nas.lan/ping"));
  expect(ping.textContent).not.toMatch(/[-]/);
  const chip = ping.querySelector<HTMLElement>("[data-template]")!;
  expect(chip).toHaveTextContent("http://nas.lan/ping");
  expect(chip).toHaveAttribute("data-template", "http://{{inputs.host}}/ping");
  act(() => {
    chip.focus();
  });
  expect(await screen.findByRole("tooltip")).toHaveTextContent("http://{{inputs.host}}/ping");
  await userEvent.click(screen.getByRole("radio", { name: "Templates" }));
  expect(ping).toHaveTextContent("GET http://{{inputs.host}}/ping");
  expect(ping.querySelector("[data-template]")).toBeNull();
  expect(window.localStorage.getItem("home-portal.workflow-editor.show-values")).toBe("0");
});
