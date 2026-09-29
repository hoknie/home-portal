import { describe, expect, it } from "vitest";

import { routes } from "@/shared/config";

import { workflowAddressOf } from "./address";

describe("workflowAddressOf", () => {
  it.each([
    ["/admin/workflows/", { kind: "list" }],
    ["/admin/workflows", { kind: "list" }],
    ["/admin/workflows/new/", { kind: "new" }],
    ["/admin/workflows/revive/", { kind: "view", id: "revive" }],
    ["/admin/workflows/revive", { kind: "view", id: "revive" }],
    ["/admin/workflows/revive-nas-2/edit/", { kind: "edit", id: "revive-nas-2" }],
    ["/admin/workflows/revive/history/", { kind: "history", id: "revive" }],
    ["/admin/workflows/revive/history/42/", { kind: "run", id: "revive", run: "42" }],
    ["/admin/workflows/revive/history/42", { kind: "run", id: "revive", run: "42" }],
    ["/admin/workflows/revive/settings/", { kind: "unknown", id: "revive" }],
    ["/admin/workflows/revive/history/42/more/", { kind: "unknown", id: "revive" }],
    ["/admin/workflows/new/edit/", { kind: "edit", id: "new" }],
  ])("%s is %j", (path, address) => {
    expect(workflowAddressOf(path)).toEqual(address);
  });

  it("every route builder gives back its own address", () => {
    expect(workflowAddressOf(routes.workflow("revive"))).toEqual({ kind: "view", id: "revive" });
    expect(workflowAddressOf(routes.workflowEdit("revive"))).toEqual({ kind: "edit", id: "revive" });
    expect(workflowAddressOf(routes.workflowHistory("revive"))).toEqual({ kind: "history", id: "revive" });
    expect(workflowAddressOf(routes.workflowRun("revive", "7"))).toEqual({ kind: "run", id: "revive", run: "7" });
    expect(workflowAddressOf(routes.newWorkflow)).toEqual({ kind: "new" });
  });

  it("a path outside the workflows is the list, so the host never guesses", () => {
    expect(workflowAddressOf("/admin/automations/")).toEqual({ kind: "list" });
  });
});
