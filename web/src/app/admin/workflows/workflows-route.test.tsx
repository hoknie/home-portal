import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";

import { pushAddress } from "@/shared/lib/navigation";

import { WorkflowsRoute } from "./workflows-route";

vi.mock("next/navigation", () => ({ usePathname: () => null }));

vi.mock("@/widgets/workflows", () => ({ WorkflowsScreen: () => <p>list</p> }));

vi.mock("@/widgets/workflow-editor", () => ({
  WorkflowEditorScreen: ({ mode, id, template, lastShownRun }: { mode: string; id: string | null; template: string | null; lastShownRun: string | null }) => (
    <p>
      {`editor ${mode} ${id ?? "-"} ${template ?? "-"} ${lastShownRun ?? "-"}`}
    </p>
  ),
  WorkflowPage: ({ id, view, run, onRunShown }: { id: string; view: string; run: string | null; onRunShown: (workflow: string, run: string) => void }) => (
    <button type="button" onClick={() => onRunShown(id, run ?? "newest")}>
      {`page ${id} ${view} ${run ?? "-"}`}
    </button>
  ),
}));

beforeEach(() => {
  window.history.replaceState(null, "", "/admin/workflows/");
});

function at(path: string) {
  window.history.replaceState(null, "", path);
  return render(<WorkflowsRoute />);
}

it.each([
  ["/admin/workflows/", "list"],
  ["/admin/workflows/new/?template=retry", "editor new - retry -"],
  ["/admin/workflows/revive/", "page revive view -"],
  ["/admin/workflows/revive/edit/", "editor edit revive - -"],
  ["/admin/workflows/revive/history/", "page revive history -"],
  ["/admin/workflows/revive/history/42/", "page revive run 42"],
  ["/admin/workflows/revive/settings/", "page revive view -"],
])("%s shows %s", (path, shown) => {
  at(path);
  expect(screen.getByText(shown)).toBeInTheDocument();
});

it("moving between addresses changes the screen, and the editor gets the run last shown", () => {
  at("/admin/workflows/revive/history/42/");
  act(() => {
    screen.getByRole("button", { name: "page revive run 42" }).click();
  });
  act(() => {
    pushAddress("/admin/workflows/revive/edit/");
  });
  expect(screen.getByText("editor edit revive - 42")).toBeInTheDocument();
  act(() => {
    pushAddress("/admin/workflows/");
  });
  expect(screen.getByText("list")).toBeInTheDocument();
});

