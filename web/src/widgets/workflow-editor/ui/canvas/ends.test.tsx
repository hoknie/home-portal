import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { inspector, node, openEditor, pressOnCanvas, stubCanvasDom, withSteps } from "../testing-support";
import { railPath, slotSpot } from "./flow-edge";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

beforeEach(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

async function slot(key: string) {
  return waitFor(() => {
    const found = document.querySelector<HTMLElement>(`[data-slot="${key}"]`);
    expect(found, key).not.toBeNull();
    return found as HTMLElement;
  });
}

function markers() {
  return screen.queryAllByRole("note").map((marker) => marker.textContent);
}

const inLoop = withSteps([
  {
    id: "each",
    kind: "loop",
    repeat: 2,
    body: [{ id: "check", kind: "if", condition: { left: "a", op: "==", right: "b" }, then: [], else: [{ id: "pause", kind: "wait", seconds: 1 }] }],
  },
]);

describe("a rail", () => {
  it("runs straight down its own column and turns only at the rail level", () => {
    expect(railPath({ x: 100, y: 0 }, { x: 300, y: 400 }, 350)).toBe("M 100 0 L 100 336 Q 100 350 114 350 L 286 350 Q 300 350 300 364 L 300 400");
    expect(railPath({ x: 100, y: 0 }, { x: 100, y: 400 }, 350)).toBe("M 100 0 L 100 400");
  });

  it("puts its + right under the branch's last step, never below the rail", () => {
    expect(slotSpot({ x: 100, y: 0 }, { x: 300, y: 400 }, 350)).toEqual({ x: 100, y: 48 });
    expect(slotSpot({ x: 100, y: 330 }, { x: 300, y: 400 }, 350)).toEqual({ x: 100, y: 350 });
    expect(slotSpot({ x: 100, y: 0 }, { x: 100, y: 100 }, undefined)).toEqual({ x: 100, y: 50 });
  });
});

it("a branch that ends with a stop shows its own end, and the shared end follows the other branch", async () => {
  openEditor(
    withSteps([
      {
        id: "check",
        kind: "if",
        condition: { left: "a", op: "==", right: "b" },
        then: [{ id: "fail", kind: "stop", outcome: "failed" }],
        else: [{ id: "tell", kind: "notify", text: "x" }],
      },
      { id: "note", kind: "log", message: "x" },
    ]),
  );
  await node("note");
  expect(markers()).toEqual(["End: failed"]);
  expect(await node("End")).toBeInTheDocument();
});

it("a step after a stop is dimmed and warned about", async () => {
  openEditor(
    withSteps([
      { id: "done", kind: "stop", outcome: "succeeded" },
      { id: "ask", kind: "http", url: "http://nas.lan" },
    ]),
  );
  const late = await node("ask");
  expect(late).toHaveAttribute("data-unreachable", "true");
  expect(within(late).getByTitle("1 warning")).toBeInTheDocument();
  expect(screen.queryByRole("group", { name: "End" })).toBeNull();
});

it("the + in a branch inside a loop offers the quick ends, and leave loop inserts a break with its marker", async () => {
  openEditor(inLoop);
  pressOnCanvas(await slot("steps[0].body[0]|then|0"));
  const dialog = await screen.findByRole("dialog", { name: "Add a step" });
  const quick = within(dialog).getByRole("region", { name: "End this branch" });
  expect(
    within(quick)
      .getAllByRole("button")
      .map((button) => button.textContent),
  ).toEqual(["End run", "Skip", "Leave loop", "Next pass"]);
  await userEvent.click(within(quick).getByRole("button", { name: "Leave loop" }));
  expect((await node("break")).getAttribute("data-path")).toBe("steps[0].body[0].then[0]");
  expect(markers()).toEqual(["Leave loop"]);
  document.body.focus();
  await userEvent.keyboard("{Control>}z{/Control}");
  await waitFor(() => expect(screen.queryByRole("group", { name: "break" })).toBeNull());
  expect(markers()).toEqual([]);
});

it("end run inserts a succeeding stop and opens its inspector to choose the outcome", async () => {
  openEditor(inLoop);
  pressOnCanvas(await slot("steps[0].body[0]|then|0"));
  const dialog = await screen.findByRole("dialog", { name: "Add a step" });
  await userEvent.click(within(dialog).getByRole("button", { name: "End run" }));
  expect(await node("stop")).toBeInTheDocument();
  expect(within(inspector()).getByText("Ends the whole run as succeeded or failed, with a reason.")).toBeInTheDocument();
  expect(markers()).toEqual(["End: succeeded"]);
});

it("the + at the top level offers no quick ends and no loop exits", async () => {
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }]));
  pressOnCanvas(await slot("|steps|1"));
  const dialog = await screen.findByRole("dialog", { name: "Add a step" });
  expect(within(dialog).queryByRole("region", { name: "End this branch" })).toBeNull();
  expect(within(dialog).queryByText("Leave loop")).toBeNull();
  expect(within(dialog).queryByText("Next pass")).toBeNull();
  expect(within(dialog).getByText("Stop")).toBeInTheDocument();
});

it("a parallel branch inside a loop offers ends but not loop exits", async () => {
  openEditor(
    withSteps([{ id: "each", kind: "loop", repeat: 2, body: [{ id: "fan", kind: "parallel", branches: [[], [{ id: "pause", kind: "wait", seconds: 1 }]] }] }]),
  );
  pressOnCanvas(await slot("steps[0].body[0]|branches[0]|0"));
  const quick = within(await screen.findByRole("dialog", { name: "Add a step" })).getByRole("region", { name: "End this branch" });
  expect(
    within(quick)
      .getAllByRole("button")
      .map((button) => button.textContent),
  ).toEqual(["End run", "Skip"]);
});
