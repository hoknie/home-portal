import type { TraceEntry } from "@/entities/automation/@x/workflow";

import { pathText } from "../tree";
import type { Flow } from "./build";
import { END_ID, START_ID } from "./layout";

export type NodeRun = {
  outcome: TraceEntry["outcome"];
  passes: number;
  durationMilliseconds: number;
  running: boolean;
  branch: string | null;
  entries: TraceEntry[];
};

export type Overlay = Map<string, NodeRun>;

export function overlayOf(entries: TraceEntry[]): Overlay {
  const overlay: Overlay = new Map();
  for (const entry of entries) {
    const current = overlay.get(entry.path);
    const running = entry.outcome === "running";
    if (current === undefined) {
      overlay.set(entry.path, {
        outcome: entry.outcome,
        passes: 1,
        durationMilliseconds: entry.duration_milliseconds,
        running,
        branch: entry.kind === "if" && entry.detail !== "" ? entry.detail : null,
        entries: [entry],
      });
      continue;
    }
    overlay.set(entry.path, {
      outcome: current.outcome === "failed" || current.outcome === "timed-out" ? current.outcome : entry.outcome,
      passes: current.passes + 1,
      durationMilliseconds: current.durationMilliseconds + entry.duration_milliseconds,
      running: current.running || running,
      branch: entry.kind === "if" && entry.detail !== "" ? entry.detail : current.branch,
      entries: [...current.entries, entry],
    });
  }
  return overlay;
}

export type RunState = { active: boolean; succeeded: boolean };

export type RunPath = { nodes: Set<string>; edges: Set<string>; order: Map<string, number> };

export function orderOf(entries: TraceEntry[]): Map<string, number> {
  const order = new Map<string, number>();
  for (const entry of entries) {
    if (!order.has(entry.path)) {
      order.set(entry.path, order.size + 1);
    }
  }
  return order;
}

export function runPath(flow: Flow, overlay: Overlay, entries: TraceEntry[], state: RunState): RunPath {
  const nodes = new Set<string>([START_ID]);
  for (const node of flow.nodes) {
    if (node.type === "step" && overlay.has(node.id)) {
      nodes.add(node.id);
    }
    if (node.type === "join" && node.path) {
      const owner = overlay.get(pathText(node.path));
      if (owner && !owner.running) {
        nodes.add(node.id);
      }
    }
  }
  if (!state.active && state.succeeded) {
    nodes.add(END_ID);
  }
  const edges = new Set<string>();
  const byId = new Map(flow.nodes.map((node) => [node.id, node]));
  for (const edge of flow.edges) {
    const source = byId.get(edge.source);
    const forked = source?.type === "step" && source.step?.kind === "if" && (edge.label?.key === "then" || edge.label?.key === "else");
    if (forked) {
      if (overlay.get(edge.source)?.branch === edge.label?.key) {
        edges.add(edge.id);
        nodes.add(edge.target);
      }
      continue;
    }
    if (edge.kind === "loop-back") {
      if ((overlay.get(edge.target)?.entries.length ?? 0) > 0 && (overlay.get(firstInside(flow, edge.target))?.passes ?? 0) > 1) {
        edges.add(edge.id);
      }
      continue;
    }
    if (nodes.has(edge.source) && nodes.has(edge.target)) {
      edges.add(edge.id);
    }
  }
  return { nodes, edges, order: orderOf(entries) };
}

function firstInside(flow: Flow, loop: string) {
  return flow.edges.find((edge) => edge.source === loop && edge.kind === "flow")?.target ?? "";
}
