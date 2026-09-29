import { type Flow, GAP_Y, type Path, type Target, contains, pathText } from "@/entities/workflow";

export const DROP_RADIUS = 48;

export type SlotPoint = { target: Target; x: number; y: number };

export function slotPoints(flow: Flow): SlotPoint[] {
  const boxes = new Map(flow.nodes.map((node) => [node.id, node.box]));
  const points: SlotPoint[] = [];
  for (const edge of flow.edges) {
    const source = boxes.get(edge.source);
    const target = boxes.get(edge.target);
    if (edge.slot && source && target) {
      const bottom = source.y + source.height;
      points.push(
        edge.rail === undefined
          ? { target: edge.slot, x: target.x + target.width / 2, y: (bottom + target.y) / 2 }
          : { target: edge.slot, x: source.x + source.width / 2, y: Math.min(bottom + GAP_Y / 2, Math.max(edge.rail, bottom)) },
      );
    }
  }
  for (const node of flow.nodes) {
    if (node.target) {
      points.push({ target: node.target, x: node.box.x + node.box.width / 2, y: node.box.y + node.box.height / 2 });
    }
  }
  return points;
}

function sameSpot(from: Path, target: Target) {
  const last = from[from.length - 1];
  const owner = from.slice(0, -1);
  const sameOwner = owner.length === target.owner.length && owner.every((place, index) => place.list === target.owner[index].list && place.index === target.owner[index].index);
  return sameOwner && last.list === target.list && (target.index === last.index || target.index === last.index + 1);
}

export function dropTarget(
  points: SlotPoint[],
  from: Path,
  point: { x: number; y: number },
  radius = DROP_RADIUS,
  allowed: (target: Target) => boolean = () => true,
): Target | null {
  let best: { target: Target; distance: number } | null = null;
  for (const candidate of points) {
    if (contains(from, candidate.target.owner) || sameSpot(from, candidate.target) || !allowed(candidate.target)) {
      continue;
    }
    const distance = Math.hypot(candidate.x - point.x, candidate.y - point.y);
    if (distance <= radius && (best === null || distance < best.distance)) {
      best = { target: candidate.target, distance };
    }
  }
  return best?.target ?? null;
}

export function slotKey(target: Target) {
  return `${pathText(target.owner)}|${target.list}|${target.index}`;
}
