import type { Step } from "../schema";
import { type Path, ROOT, type Target, childList, listsOf, pathText } from "../tree";
import { END_ID, type Layout, START_ID, emptyId, frameId, joinId, layoutOf } from "./layout";
import type { Box } from "./sizes";

export const NODE_TYPES = ["start", "step", "join", "frame", "empty", "end"] as const;

export type FlowNodeType = (typeof NODE_TYPES)[number];

export type EdgeLabel = { key: "then" | "else" | "branch" | "repeat" | "repeatTemplate" | "forEach" | "while" | "again"; params?: Record<string, string | number> };

export type FlowNode = { id: string; type: FlowNodeType; box: Box; path?: Path; step?: Step; target?: Target };

export type FlowEdge = { id: string; source: string; target: string; kind: "flow" | "loop-back"; label?: EdgeLabel; slot?: Target };

export type Flow = { nodes: FlowNode[]; edges: FlowEdge[]; slots: Target[] };

type Builder = { layout: Layout; nodes: FlowNode[]; edges: FlowEdge[] };

function boxOf(builder: Builder, id: string): Box {
  return builder.layout.get(id) ?? { x: 0, y: 0, width: 0, height: 0 };
}

function node(builder: Builder, entry: Omit<FlowNode, "box">) {
  builder.nodes.push({ ...entry, box: boxOf(builder, entry.id) });
}

function edge(builder: Builder, source: string, target: string, extra: Partial<FlowEdge> = {}) {
  builder.edges.push({ id: `${source}->${target}`, source, target, kind: "flow", ...extra });
}

export function loopLabel(step: Step): EdgeLabel {
  if (step.for_each !== undefined) {
    return { key: "forEach", params: { list: step.for_each } };
  }
  if (step.while !== undefined) {
    return { key: "while" };
  }
  return typeof step.repeat === "string" ? { key: "repeatTemplate", params: { count: step.repeat } } : { key: "repeat", params: { count: step.repeat ?? 1 } };
}

function branchLabel(list: string): EdgeLabel {
  const branch = /^branches\[(\d+)\]$/.exec(list);
  if (branch) {
    return { key: "branch", params: { number: Number(branch[1]) + 1 } };
  }
  return { key: list === "else" ? "else" : "then" };
}

function chain(builder: Builder, steps: Step[], owner: Path, list: string, from: string, label?: EdgeLabel): string {
  if (steps.length === 0) {
    const id = emptyId(owner, list);
    const target = { owner, list, index: 0 };
    node(builder, { id, type: "empty", target });
    edge(builder, from, id, { label });
    return id;
  }
  let previous = from;
  steps.forEach((step, index) => {
    const path = [...owner, { list, index }];
    edge(builder, previous, pathText(path), { label: index === 0 ? label : undefined, slot: { owner, list, index } });
    previous = stepNode(builder, step, path);
  });
  return previous;
}

function stepNode(builder: Builder, step: Step, path: Path): string {
  const id = pathText(path);
  if (step.kind === "loop") {
    node(builder, { id: frameId(path), type: "frame", path, step });
  }
  node(builder, { id, type: "step", path, step });
  if (step.kind === "if" || step.kind === "parallel") {
    const join = joinId(path);
    node(builder, { id: join, type: "join", path });
    for (const list of listsOf(step)) {
      const steps = childList(step, list);
      const exit = chain(builder, steps, path, list, id, branchLabel(list));
      edge(builder, exit, join, steps.length === 0 ? {} : { slot: { owner: path, list, index: steps.length } });
    }
    return join;
  }
  if (step.kind === "loop") {
    const join = joinId(path);
    node(builder, { id: join, type: "join", path });
    const body = childList(step, "body");
    const exit = chain(builder, body, path, "body", id, loopLabel(step));
    edge(builder, exit, join, body.length === 0 ? {} : { slot: { owner: path, list: "body", index: body.length } });
    edge(builder, join, id, { kind: "loop-back", label: { key: "again" } });
    return join;
  }
  return id;
}

export function flowOf(steps: Step[]): Flow {
  const builder: Builder = { layout: layoutOf(steps), nodes: [], edges: [] };
  node(builder, { id: START_ID, type: "start" });
  const exit = chain(builder, steps, [], ROOT, START_ID);
  node(builder, { id: END_ID, type: "end" });
  edge(builder, exit, END_ID, steps.length === 0 ? {} : { slot: { owner: [], list: ROOT, index: steps.length } });
  const slots = [
    ...builder.edges.flatMap((entry) => (entry.slot ? [entry.slot] : [])),
    ...builder.nodes.flatMap((entry) => (entry.target ? [entry.target] : [])),
  ];
  return { nodes: builder.nodes, edges: builder.edges, slots };
}
