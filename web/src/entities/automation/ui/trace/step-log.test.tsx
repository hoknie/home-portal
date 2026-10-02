import { screen, within } from "@testing-library/react";
import { expect, it } from "vitest";

import { TRACE_ENTRY_BLANKS, renderWithProviders } from "@/shared/lib/testing";

import { type TraceEntry, traceEntrySchema } from "../../model/schema";
import { passRows } from "./pass-groups";
import { TraceTimeline } from "./trace-timeline";

function entry(fields: Partial<TraceEntry> & Pick<TraceEntry, "path" | "step" | "kind">): TraceEntry {
  return traceEntrySchema.parse({ ...TRACE_ENTRY_BLANKS, label: fields.step, iteration: null, outcome: "succeeded", started_at: "2026-09-28T19:15:59Z", duration_milliseconds: 1, detail: "", output: null, ...fields });
}

const oshof = [
  entry({ path: "steps[0]", step: "services", kind: "set", detail: 'svc = ["Media","qTorrent"]', log: ['svc = ["Media","qTorrent"]'] }),
  entry({ path: "steps[1]", step: "svc_iter", kind: "loop", outcome: "failed", detail: "no service Media", values: [{ template: "{{vars.svc}}", value: '["Media","qTorrent"]' }], log: ["for each: 2 items", 'pass 1: "Media"', "ended after 1 passes"] }),
  entry({
    path: "steps[1].body[0]",
    step: "status",
    kind: "status",
    iteration: 0,
    item: '"Media"',
    outcome: "failed",
    detail: "no service Media",
    values: [{ template: "{{loop.item}}", value: '"Media"' }],
    log: ['"Media" is the name of the service media; use {{portal.services.media.id}}', "known services: media, qtorrent"],
  }),
];

it("seeing why a status step failed: the pass, what the template gave and the hint are open", () => {
  renderWithProviders(<TraceTimeline trace={{ entries: oshof, dropped: 0, outputs: null }} />);
  const steps = screen.getByRole("list", { name: "Steps" });
  expect(within(steps).getByText('Pass 1: "Media"')).toBeInTheDocument();
  const status = within(steps).getByText("status", { selector: "span.font-medium" }).closest("li")!;
  const details = status.querySelector("details")!;
  expect(details).toHaveAttribute("open");
  const values = within(status).getByRole("region", { name: "Values" });
  expect(values).toHaveTextContent("{{loop.item}}");
  expect(values).toHaveTextContent('"Media"');
  expect(within(status).getByRole("region", { name: "Log" })).toHaveTextContent("use {{portal.services.media.id}}");
  const services = within(steps).getByText("services", { selector: "span.font-medium" }).closest("li")!;
  expect(services.querySelector("details")).not.toHaveAttribute("open");
});

it("an old entry without values or a log draws as before, without a log to open", () => {
  const old = traceEntrySchema.parse({ ...TRACE_ENTRY_BLANKS, path: "steps[0]", step: "ping", label: "ping", kind: "http", iteration: null, outcome: "failed", started_at: "", duration_milliseconds: 3, detail: "500", output: null });
  renderWithProviders(<TraceTimeline trace={{ entries: [old], dropped: 0, outputs: null }} />);
  expect(screen.getByText("500")).toBeInTheDocument();
  expect(screen.queryByText("Values and log")).toBeNull();
});

it("a log step shows its level beside its label and its message in the level's colour", () => {
  const note = entry({ path: "steps[0]", step: "note", kind: "log", level: "warning", detail: "checking nas: down", log: ["[warning] checking nas: down"] });
  renderWithProviders(<TraceTimeline trace={{ entries: [note], dropped: 0, outputs: null }} />);
  expect(screen.getAllByText("warning")[0]).toHaveClass("text-status-degraded");
});

it("passes are grouped under their loop, once per pass, also for a nested loop", () => {
  const rows = passRows([
    entry({ path: "steps[0]", step: "outer", kind: "loop" }),
    entry({ path: "steps[0].body[0]", step: "a", kind: "set", iteration: 0, item: '"x"' }),
    entry({ path: "steps[0].body[1]", step: "inner", kind: "loop", iteration: 0 }),
    entry({ path: "steps[0].body[1].body[0]", step: "b", kind: "set", iteration: 0 }),
    entry({ path: "steps[0].body[1].body[0]", step: "b", kind: "set", iteration: 1 }),
    entry({ path: "steps[0].body[0]", step: "a", kind: "set", iteration: 1, item: '"y"' }),
  ]);
  expect(rows.map((row) => (row.type === "pass" ? `pass ${row.loop} ${row.number} ${row.item ?? ""} ${row.depth}` : row.type === "branch" ? `branch ${row.number}` : row.entry.step))).toEqual([
    "outer",
    'pass steps[0] 1 "x" 1',
    "a",
    "inner",
    "pass steps[0].body[1] 1  2",
    "b",
    "pass steps[0].body[1] 2  2",
    "b",
    'pass steps[0] 2 "y" 1',
    "a",
  ]);
});

it("parallel branches are grouped one after another and a nested loop numbers its passes again in every outer pass", () => {
  const rows = passRows([
    entry({ path: "steps[0]", step: "both", kind: "parallel" }),
    entry({ path: "steps[0].branches[0][0]", step: "a1", kind: "wait" }),
    entry({ path: "steps[0].branches[1][0]", step: "b1", kind: "set" }),
    entry({ path: "steps[0].branches[0][1]", step: "a2", kind: "set" }),
    entry({ path: "steps[1]", step: "outer", kind: "loop" }),
    entry({ path: "steps[1].body[0]", step: "inner", kind: "loop", iteration: 0 }),
    entry({ path: "steps[1].body[0].body[0]", step: "c", kind: "set", iteration: 0 }),
    entry({ path: "steps[1].body[0]", step: "inner", kind: "loop", iteration: 1 }),
    entry({ path: "steps[1].body[0].body[0]", step: "c", kind: "set", iteration: 0 }),
  ]);
  expect(rows.map((row) => (row.type === "entry" ? row.entry.step : row.type === "branch" ? `branch ${row.number}` : `pass ${row.number}`))).toEqual([
    "both",
    "branch 1",
    "a1",
    "a2",
    "branch 2",
    "b1",
    "outer",
    "pass 1",
    "inner",
    "pass 1",
    "c",
    "pass 2",
    "inner",
    "pass 1",
    "c",
  ]);
});
