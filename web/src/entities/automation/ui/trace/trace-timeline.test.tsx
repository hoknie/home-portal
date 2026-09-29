import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { type Run, type TraceEntry, runsSchema } from "../../model/schema";
import { RunDetails } from "../run-details";
import { TraceTimeline, depthOf } from "./trace-timeline";

const [running] = runsSchema.parse(apiSamples.automationRuns).runs;

afterEach(() => {
  vi.unstubAllGlobals();
});

function entry(path: string, step: string, kind: string, outcome: TraceEntry["outcome"], output: string | null = null): TraceEntry {
  return { path, step, label: step, kind, iteration: null, outcome, started_at: "2026-09-25T03:00:00Z", duration_milliseconds: 20, detail: "", output, shape: null, stdout: null, stderr: null, command: null, budget_reached: false, values: [], log: [], values_dropped: 0, log_dropped: 0, item: null, level: null, wait_seconds: null };
}

function workflowRun(entries: TraceEntry[], result: Run["outcome"]["result"]): Run {
  return { ...running, workflow: "check", trace: { entries, dropped: 0 }, outcome: { ...running.outcome, result } };
}

it("indents an entry by its nesting", () => {
  expect(depthOf("steps[0]")).toBe(0);
  expect(depthOf("steps[1].then[0].body[2]")).toBe(2);
  expect(depthOf("steps[2].branches[1][0]")).toBe(2);
});

it("opens an http answer on demand, formatted, and counts steps left out", async () => {
  renderWithProviders(<TraceTimeline trace={{ entries: [entry("steps[0]", "ping", "http", "succeeded", '{"state":"up"}')], dropped: 3 }} />);
  expect(screen.getByRole("list", { name: "Answer" })).not.toBeVisible();
  await userEvent.click(screen.getByText("Answer"));
  expect(screen.getByRole("list", { name: "Answer" })).toHaveTextContent('state: "up"');
  expect(screen.queryByText(/Shortened/)).toBeNull();
  expect(screen.getByText("3 more steps are not shown")).toBeInTheDocument();
});

it("a long JSON answer is shown from its shape, formatted, folded below the top and marked as shortened", async () => {
  const cut = { ...entry("steps[0]", "list", "http", "succeeded", '{"disks":[{"name":"sd0"},{"na'), shape: '{"disks":[{"name":"sd0"},{"name":"sd1"},{"name":"sd2"}]}' };
  renderWithProviders(<TraceTimeline trace={{ entries: [cut], dropped: 0 }} />);
  await userEvent.click(screen.getByText("Answer"));
  const tree = screen.getByRole("list", { name: "Answer" });
  expect(tree).toHaveTextContent("disks: [3]");
  expect(tree.querySelector("details[data-depth='1']")).not.toHaveAttribute("open");
  expect(screen.getByText(/Shortened/)).toBeInTheDocument();
});

it("following a run shows the finished step and the running wait, then the next step without a reload", async () => {
  const first = workflowRun([entry("steps[0]", "first", "http", "succeeded"), entry("steps[1]", "pause", "wait", "running")], "running");
  const second = workflowRun(
    [entry("steps[0]", "first", "http", "succeeded"), entry("steps[1]", "pause", "wait", "succeeded"), entry("steps[2]", "second", "http", "succeeded")],
    "succeeded",
  );
  const fetch = vi.fn().mockResolvedValueOnce(jsonResponse(first)).mockImplementation(async () => jsonResponse(second));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<RunDetails runId="5" onClose={() => {}} />);
  const steps = await screen.findByRole("list", { name: "Steps" });
  expect(within(steps).getByText("first").closest("li")).toHaveAttribute("data-outcome", "succeeded");
  expect(within(steps).getByText("pause").closest("li")).toHaveAttribute("data-outcome", "running");
  expect(within(steps).queryByText("second")).toBeNull();
  expect(await within(steps).findByText("second", {}, { timeout: 3000 })).toBeInTheDocument();
  expect(within(steps).getByText("pause").closest("li")).toHaveAttribute("data-outcome", "succeeded");
});

it("the log from the journal: a script entry opens its log from Details instead of an Output line", async () => {
  const failed = {
    ...entry("steps[0]", "restart", "script", "failed"),
    detail: "restart.sh exited 1: container not found",
    stdout: { tail: "stopping\n", bytes: 9, truncated: false },
    stderr: { tail: "container not found\n", bytes: 20, truncated: false },
    command: ["restart.sh", "jellyfin"],
  };
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(workflowRun([failed], "failed"))));
  renderWithProviders(<RunDetails runId="5" onClose={() => {}} />);
  const steps = await screen.findByRole("list", { name: "Steps" });
  expect(within(steps).getByText("restart.sh exited 1: container not found")).toBeInTheDocument();
  expect(within(steps).queryByText("Output")).toBeNull();
  await userEvent.click(within(steps).getByRole("button", { name: "Details" }));
  const dialog = await screen.findByRole("dialog", { name: "Log of restart" });
  expect(within(dialog).getByText("restart.sh 'jellyfin'")).toBeInTheDocument();
  expect(within(dialog).getByText("Standard error").parentElement).toHaveTextContent("container not found");
});

it("the detail under a step's name stays on one line, with the whole text on hover", () => {
  const long = `restart.sh exited 1: ${"x".repeat(200)}`;
  renderWithProviders(<TraceTimeline trace={{ entries: [{ ...entry("steps[0].then[0]", "restart", "script", "failed"), detail: long }], dropped: 0 }} indent={false} />);
  const detail = document.querySelector("[data-detail]")!;
  expect(detail.className).toContain("truncate");
  expect(detail).toHaveAttribute("title", long);
  expect(detail.closest("li")).not.toHaveAttribute("style");
});
