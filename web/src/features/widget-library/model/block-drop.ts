import type { RawBlock } from "./blocks";
import { type BlockPath, type DropTarget, blockAt, childrenOf, insertAt, isGroup, isSlot, moveTo, removeAt, updateAt } from "./block-tree";

export const CANVAS_ATTRIBUTE = "data-block-path";

export const OUTLINE_ATTRIBUTE = "data-outline-path";

export const SLOT_ATTRIBUTE = "data-outline-slot";

export const CANVAS_ROOT = "data-canvas-root";

export type DragSource = { kind: "new"; block: RawBlock } | { kind: "move"; path: BlockPath };

export type Box = { top: number; left: number; width: number; height: number };

export type Drop = { target: DropTarget; replace: boolean; box: Box; mode: "line-x" | "line-y" | "box" };

export function pathOf(text: string | null): BlockPath | null {
  if (text === null) {
    return null;
  }
  return text === "" ? [] : text.split(".").map(Number);
}

export function textOf(path: BlockPath): string {
  return path.join(".");
}

function boxOf(element: Element): Box {
  const rect = element.getBoundingClientRect();
  return { top: rect.top, left: rect.left, width: rect.width, height: rect.height };
}


function edge(box: Box, after: boolean, horizontal: boolean): Box {
  if (horizontal) {
    return { top: box.top, left: after ? box.left + box.width : box.left, width: 0, height: box.height };
  }
  return { top: after ? box.top + box.height : box.top, left: box.left, width: box.width, height: 0 };
}

function deepest(candidates: Element[]): Element | null {
  return candidates.find((candidate) => candidates.every((other) => other === candidate || !candidate.contains(other))) ?? null;
}

function between(container: Element, blocks: RawBlock[], parent: BlockPath, horizontal: boolean, x: number, y: number): Drop {
  const prefix = parent.length === 0 ? "" : `${textOf(parent)}.`;
  const children = [...container.querySelectorAll(`[${CANVAS_ATTRIBUTE}]`)].filter((element) => {
    const text = element.getAttribute(CANVAS_ATTRIBUTE) ?? "";
    return text.startsWith(prefix) && !text.slice(prefix.length).includes(".");
  });
  const owner = parent.length === 0 ? blocks : childrenOf(blockAt(blocks, parent) as RawBlock);
  const boxes = children.map(boxOf);
  const index = boxes.findIndex((box) => (horizontal ? x < box.left + box.width / 2 : y < box.top + box.height / 2));
  const mode = horizontal ? "line-x" : "line-y";
  if (boxes.length === 0) {
    return { target: { parent, index: owner.length }, replace: false, box: boxOf(container), mode: "box" };
  }
  if (index < 0) {
    return { target: { parent, index: owner.length }, replace: false, box: edge(boxes[boxes.length - 1], true, horizontal), mode };
  }
  return { target: { parent, index }, replace: false, box: edge(boxes[index], false, horizontal), mode };
}

export function dropAt(blocks: RawBlock[], element: Element | null, x: number, y: number): Drop | null {
  const found = [SLOT_ATTRIBUTE, OUTLINE_ATTRIBUTE, CANVAS_ATTRIBUTE, CANVAS_ROOT].map((attribute) => element?.closest(`[${attribute}]`)).filter((candidate): candidate is Element => Boolean(candidate));
  const hit = deepest(found);
  if (!hit) {
    return null;
  }
  if (hit.hasAttribute(CANVAS_ROOT)) {
    return between(hit, blocks, [], false, x, y);
  }
  if (hit.hasAttribute(SLOT_ATTRIBUTE)) {
    const parent = pathOf(hit.getAttribute(SLOT_ATTRIBUTE)) ?? [];
    const owner = parent.length === 0 ? blocks : childrenOf(blockAt(blocks, parent) as RawBlock);
    return { target: { parent, index: owner.length }, replace: false, box: boxOf(hit), mode: "box" };
  }
  const outlined = hit.hasAttribute(OUTLINE_ATTRIBUTE);
  const path = pathOf(hit.getAttribute(outlined ? OUTLINE_ATTRIBUTE : CANVAS_ATTRIBUTE));
  if (path === null || path.length === 0) {
    return null;
  }
  const block = blockAt(blocks, path);
  const parent = path.slice(0, -1);
  const index = path[path.length - 1];
  const box = boxOf(hit);
  if (!outlined && isGroup(block)) {
    return between(hit, blocks, path, block?.kind === "row", x, y);
  }
  if (isSlot(block) && parent.length > 0) {
    return { target: { parent, index }, replace: true, box, mode: "box" };
  }
  if (outlined && isGroup(block) && y > box.top + box.height / 4 && y < box.top + (box.height * 3) / 4) {
    return { target: { parent: path, index: childrenOf(block as RawBlock).length }, replace: false, box, mode: "box" };
  }
  const horizontal = !outlined && parent.length > 0 && blockAt(blocks, parent)?.kind === "row";
  const after = horizontal ? x > box.left + box.width / 2 : y > box.top + box.height / 2;
  return { target: { parent, index: index + (after ? 1 : 0) }, replace: false, box: edge(box, after, horizontal), mode: horizontal ? "line-x" : "line-y" };
}

export function dropped(blocks: RawBlock[], source: DragSource, drop: Drop): [RawBlock[], BlockPath] {
  if (source.kind === "new") {
    if (drop.replace) {
      const path = [...drop.target.parent, drop.target.index];
      return [updateAt(blocks, path, source.block), path];
    }
    return [insertAt(blocks, drop.target, source.block), [...drop.target.parent, drop.target.index]];
  }
  const [moved, landed] = moveTo(blocks, source.path, drop.target);
  if (!drop.replace || moved === blocks) {
    return [moved, landed];
  }
  return [removeAt(moved, [...landed.slice(0, -1), landed[landed.length - 1] + 1]), landed];
}
