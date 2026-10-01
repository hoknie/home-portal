import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { type Step, workflowCatalogueSchema } from "./schema";
import { at, duplicate, idFor, idsOf, insert, move, moveBy, newStep, parsePath, pathText, remove, stepsIn, updateAt } from "./tree";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);

function tree(): Step[] {
  return [
    { id: "ping", kind: "http", method: "GET", url: "http://nas.lan/api/ping" },
    {
      id: "check",
      kind: "if",
      condition: { left: "{{steps.ping.status}}", op: "==", right: "200" },
      then: [{ id: "ok", kind: "stop", outcome: "succeeded" }],
      else: [],
    },
    { id: "both", kind: "parallel", branches: [[{ id: "a", kind: "status", service: "nas" }], [{ id: "b", kind: "status", service: "router" }]] },
    { id: "tell", kind: "notify", text: "done" },
  ];
}

describe("workflow tree", () => {
  it("paths read and write like the server's error paths", () => {
    const parsed = parsePath("steps[1].then[0].url");
    expect(parsed).toEqual({ path: [{ list: "steps", index: 1 }, { list: "then", index: 0 }], field: "url" });
    expect(pathText(parsed.path)).toBe("steps[1].then[0]");
    expect(at(tree(), parsed.path)?.id).toBe("ok");
    const branch = parsePath("steps[2].branches[1][0].service");
    expect(branch.field).toBe("service");
    expect(at(tree(), branch.path)?.id).toBe("b");
    expect(parsePath("title")).toEqual({ path: [], field: "title" });
  });

  it("insert, update and remove work in nested lists", () => {
    const inserted = insert(tree(), { owner: [{ list: "steps", index: 1 }], list: "else", index: 0 }, { id: "bad", kind: "stop", outcome: "failed" });
    expect(at(inserted, parsePath("steps[1].else[0]").path)?.id).toBe("bad");
    const changed = updateAt(inserted, parsePath("steps[1].else[0]").path, (step) => ({ ...step, reason: "down" }));
    expect(at(changed, parsePath("steps[1].else[0]").path)?.reason).toBe("down");
    const removed = remove(changed, parsePath("steps[1].then[0]").path);
    expect(stepsIn(removed, [{ list: "steps", index: 1 }], "then")).toEqual([]);
    expect(tree()[1].then).toHaveLength(1);
  });

  it("moving within a list and across lists keeps the other steps in place", () => {
    const down = moveBy(tree(), [{ list: "steps", index: 0 }], 1);
    expect(down.map((step) => step.id)).toEqual(["check", "ping", "both", "tell"]);
    expect(moveBy(tree(), [{ list: "steps", index: 0 }], -1)).toEqual(tree());
    const into = move(tree(), [{ list: "steps", index: 0 }], { owner: [{ list: "steps", index: 1 }], list: "then", index: 0 });
    expect(into.map((step) => step.id)).toEqual(["check", "both", "tell"]);
    expect(stepsIn(into, [{ list: "steps", index: 0 }], "then").map((step) => step.id)).toEqual(["ping", "ok"]);
    const later = move(tree(), [{ list: "steps", index: 0 }], { owner: [], list: "steps", index: 3 });
    expect(later.map((step) => step.id)).toEqual(["check", "both", "ping", "tell"]);
    const out = move(tree(), parsePath("steps[2].branches[0][0]").path, { owner: [], list: "steps", index: 0 });
    expect(out[0].id).toBe("a");
    expect(out[3].branches?.[0]).toEqual([]);
  });

  it("a block cannot move into itself", () => {
    expect(move(tree(), [{ list: "steps", index: 1 }], { owner: [{ list: "steps", index: 1 }], list: "then", index: 0 })).toEqual(tree());
  });

  it("duplicating gives the copy and its children fresh ids", () => {
    const copied = duplicate(tree(), [{ list: "steps", index: 1 }]);
    expect(copied.map((step) => step.id)).toEqual(["ping", "check", "check_2", "both", "tell"]);
    expect(copied[2].then?.[0].id).toBe("ok_2");
    expect(idsOf(copied).size).toBe(idsOf(tree()).size + 2);
  });

  it("an id follows a label and avoids taken ids", () => {
    expect(idFor("Ping the NAS", new Set())).toBe("ping_the_nas");
    expect(idFor("Ping", new Set(["ping", "ping_2"]))).toBe("ping_3");
    expect(idFor("42", new Set())).toBe("step");
  });

  it("a new step gets the fields its kind requires", () => {
    const kind = (name: string) => catalogue.kinds.find((entry) => entry.name === name)!;
    expect(newStep(kind("if"), new Set())).toMatchObject({ id: "if", kind: "if", then: [], condition: { op: "==" } });
    expect(newStep(kind("loop"), new Set())).toMatchObject({ kind: "loop", repeat: 1, body: [] });
    expect(newStep(kind("parallel"), new Set(["parallel"]))).toMatchObject({ id: "parallel_2", branches: [[], []] });
    expect(newStep(kind("http"), new Set())).toMatchObject({ method: "GET", url: "" });
  });
});
