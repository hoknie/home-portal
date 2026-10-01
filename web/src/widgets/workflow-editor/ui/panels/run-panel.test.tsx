import { fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { runsSchema, traceSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { TRACE_ENTRY_BLANKS, jsonResponse } from "@/shared/lib/testing";

import { node, openAt, stubCanvasDom, withSteps } from "../testing-support";

vi.mock("next/navigation", () => ({ usePathname: () => null, useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

beforeEach(() => {
  stubCanvasDom();
  window.localStorage.setItem("home-portal.workflow-editor.legend-dismissed", "1");
});

afterEach(() => {
  vi.unstubAllGlobals();
  window.localStorage.clear();
});

it("the steps of the run and the chosen step's data are two headed parts, and a longer list says there is more below", async () => {
  const [base] = runsSchema.parse(apiSamples.automationRuns).runs;
  const steps = ["one", "two", "three"].map((id, index) => ({ ...TRACE_ENTRY_BLANKS, path: `steps[${index}]`, step: id, label: id, kind: "wait", iteration: null, outcome: "succeeded", started_at: "2026-09-29T10:00:00Z", duration_milliseconds: 5, detail: "", output: null }));
  const shown = { ...base, id: "9", workflow: "draft", trace: traceSchema.parse({ entries: steps, dropped: 0 }) };
  vi.stubGlobal("fetch", vi.fn(async (path: string) => (path === "/api/automations/runs/9" ? jsonResponse(shown) : jsonResponse({ runs: [shown], next_before: null }))));
  openAt(withSteps(["one", "two", "three"].map((id) => ({ id, kind: "wait", seconds: 1 })), { last_run: null, active_run: null }), "/admin/workflows/draft/history/9/");
  const panel = await screen.findByRole("complementary", { name: "Run" });
  expect(await within(panel).findByRole("heading", { name: "3 steps" })).toBeInTheDocument();
  const list = within(panel).getByRole("list", { name: "Steps of the run" });
  expect(within(panel).queryByRole("button", { name: "More steps below" })).toBeNull();
  Object.defineProperties(list, { scrollHeight: { configurable: true, value: 400 }, clientHeight: { configurable: true, value: 256 }, scrollTop: { configurable: true, writable: true, value: 0 } });
  fireEvent.scroll(list);
  expect(within(panel).getByRole("button", { name: "More steps below" })).toBeInTheDocument();
  list.scrollTop = 144;
  fireEvent.scroll(list);
  expect(within(panel).queryByRole("button", { name: "More steps below" })).toBeNull();
  fireEvent.click(await node("two"));
  const details = within(panel).getByRole("region", { name: "Step details" });
  expect(within(details).getByRole("heading")).toHaveTextContent("Step detailstwo");
});
