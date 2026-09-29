import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import type { TraceEntry } from "../../model/schema";
import { ScriptLogDialog } from "./script-log-dialog";

function script(overrides: Partial<TraceEntry>): TraceEntry {
  return {
    path: "steps[0]",
    step: "restart",
    label: "restart",
    kind: "script",
    iteration: null,
    outcome: "failed",
    started_at: "2026-09-25T03:00:00Z",
    duration_milliseconds: 340,
    detail: "restart.sh exited 1: container not found",
    output: null,
    shape: null,
    stdout: { tail: "stopping\n", bytes: 9, truncated: false },
    stderr: { tail: "container not found\n", bytes: 20, truncated: false },
    command: ["restart.sh", "jellyfin"],
    budget_reached: false,
    values: [],
    log: [],
    values_dropped: 0,
    log_dropped: 0,
    item: null,
    level: null,
    wait_seconds: null,
    ...overrides,
  };
}

async function opened(entry: TraceEntry) {
  renderWithProviders(<ScriptLogDialog entry={entry} />);
  await userEvent.click(screen.getByRole("button", { name: "Details" }));
  return screen.findByRole("dialog");
}

it("shows the quoted command, the outcome, the exit code, the duration and both streams", async () => {
  const dialog = await opened(script({}));
  expect(within(dialog).getByText("restart.sh 'jellyfin'")).toBeInTheDocument();
  expect(within(dialog).getByText("Exit code").nextElementSibling).toHaveTextContent("1");
  expect(within(dialog).getByText("340 ms")).toBeInTheDocument();
  expect(within(dialog).getByText("Outcome").nextElementSibling).toHaveTextContent("Failed");
  expect(within(dialog).getByText("Standard output").parentElement?.parentElement).toHaveTextContent("stopping");
  expect(within(dialog).getByText("Standard error").parentElement?.parentElement).toHaveTextContent("container not found");
});

it("an entry from before the streams were kept apart shows its single output as Output", async () => {
  const dialog = await opened(script({ stdout: null, stderr: null, command: null, output: "progress\ncontainer not found\n", detail: "restart.sh exited 1" }));
  expect(within(dialog).getByText("Output").parentElement?.parentElement).toHaveTextContent("container not found");
  expect(within(dialog).queryByText("Standard error")).toBeNull();
  expect(within(dialog).queryByText("Command")).toBeNull();
});

it("a redrawn line is shown once in its last state and a long word wraps instead of scrolling sideways", async () => {
  const meter = Array.from({ length: 100 }, (_, index) => `${index + 1}%`).join("\r");
  const long = "x".repeat(300);
  const dialog = await opened(script({ stdout: { tail: `${meter}\n${long}\n`, bytes: 1000, truncated: false } }));
  const block = within(dialog).getByText("Standard output").parentElement?.querySelector("pre") as HTMLElement;
  expect(block.textContent).toBe(`100%\n${long}\n`);
  expect(block.className).toContain("whitespace-pre-wrap");
  expect(block.className).toContain("break-all");
});

it("notes a shortened stream and a reached budget", async () => {
  const dialog = await opened(script({ stdout: { tail: "x".repeat(1024), bytes: 40960, truncated: true }, budget_reached: true }));
  expect(within(dialog).getByText(/^The last 1 KiB of 40,?960 bytes$/)).toBeInTheDocument();
  expect(within(dialog).getAllByText(/output budget was reached/)).toHaveLength(2);
});

it("copy copies the text as shown", async () => {
  const user = userEvent.setup();
  const writeText = vi.spyOn(navigator.clipboard, "writeText");
  renderWithProviders(<ScriptLogDialog entry={script({ stderr: { tail: "\u001b[31m0%\r50%\rdone\u001b[0m\n", bytes: 30, truncated: false } })} />);
  await user.click(screen.getByRole("button", { name: "Details" }));
  const dialog = await screen.findByRole("dialog");
  const errors = within(dialog).getByText("Standard error").parentElement as HTMLElement;
  await user.click(within(errors).getByRole("button", { name: "Copy" }));
  await waitFor(() => expect(writeText).toHaveBeenCalledWith("done\n"));
});
