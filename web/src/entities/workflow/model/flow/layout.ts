import type { Step } from "../schema";
import { type Path, ROOT, childList, listsOf, pathText } from "../tree";
import { type Box, CARD, EMPTY, END, FRAME_PADDING, GAP_X, GAP_Y, JOIN, type Size, TERMINAL } from "./sizes";

export type Layout = Map<string, Box>;

export const START_ID = "start";
export const END_ID = "end";

export function joinId(path: Path) {
  return `${pathText(path)}:join`;
}

export function frameId(path: Path) {
  return `${pathText(path)}:frame`;
}

export function emptyId(owner: Path, list: string) {
  return `${owner.length === 0 ? ROOT : pathText(owner)}:${list}:empty`;
}

function branchesOf(step: Step): string[] {
  return step.kind === "if" || step.kind === "parallel" ? listsOf(step) : [];
}

function listSize(steps: Step[], owner: Path, list: string): Size {
  if (steps.length === 0) {
    return EMPTY;
  }
  const sizes = steps.map((step, index) => stepSize(step, [...owner, { list, index }]));
  return {
    width: Math.max(...sizes.map((size) => size.width)),
    height: sizes.reduce((sum, size) => sum + size.height, 0) + GAP_Y * (sizes.length - 1),
  };
}

function stepSize(step: Step, path: Path): Size {
  const branches = branchesOf(step);
  if (branches.length > 0) {
    const columns = branches.map((list) => listSize(childList(step, list), path, list));
    const width = columns.reduce((sum, size) => sum + size.width, 0) + GAP_X * (columns.length - 1);
    return {
      width: Math.max(CARD.width, width),
      height: CARD.height + GAP_Y + Math.max(...columns.map((size) => size.height)) + GAP_Y + JOIN.height,
    };
  }
  if (step.kind === "loop") {
    const body = listSize(childList(step, "body"), path, "body");
    const frame = { width: body.width + FRAME_PADDING * 2, height: body.height + GAP_Y + JOIN.height + FRAME_PADDING * 2 };
    return { width: Math.max(CARD.width, frame.width), height: CARD.height + GAP_Y / 2 + frame.height };
  }
  return CARD;
}

function put(layout: Layout, id: string, center: number, top: number, size: Size) {
  layout.set(id, { x: center - size.width / 2, y: top, width: size.width, height: size.height });
}

function placeList(layout: Layout, steps: Step[], owner: Path, list: string, center: number, top: number) {
  if (steps.length === 0) {
    put(layout, emptyId(owner, list), center, top, EMPTY);
    return top + EMPTY.height;
  }
  let y = top;
  steps.forEach((step, index) => {
    y = placeStep(layout, step, [...owner, { list, index }], center, y) + (index < steps.length - 1 ? GAP_Y : 0);
  });
  return y;
}

function placeStep(layout: Layout, step: Step, path: Path, center: number, top: number): number {
  const id = pathText(path);
  put(layout, id, center, top, CARD);
  const below = top + CARD.height;
  const branches = branchesOf(step);
  if (branches.length > 0) {
    const columns = branches.map((list) => ({ list, size: listSize(childList(step, list), path, list) }));
    const total = columns.reduce((sum, column) => sum + column.size.width, 0) + GAP_X * (columns.length - 1);
    let left = center - total / 2;
    let bottom = below + GAP_Y;
    for (const column of columns) {
      const end = placeList(layout, childList(step, column.list), path, column.list, left + column.size.width / 2, below + GAP_Y);
      bottom = Math.max(bottom, end);
      left += column.size.width + GAP_X;
    }
    put(layout, joinId(path), center, bottom + GAP_Y, JOIN);
    return bottom + GAP_Y + JOIN.height;
  }
  if (step.kind === "loop") {
    const body = childList(step, "body");
    const size = listSize(body, path, "body");
    const frameTop = below + GAP_Y / 2;
    const end = placeList(layout, body, path, "body", center, frameTop + FRAME_PADDING);
    put(layout, joinId(path), center, end + GAP_Y, JOIN);
    const frameBottom = end + GAP_Y + JOIN.height + FRAME_PADDING;
    put(layout, frameId(path), center, frameTop, { width: size.width + FRAME_PADDING * 2, height: frameBottom - frameTop });
    return frameBottom;
  }
  return below;
}

export function layoutOf(steps: Step[]): Layout {
  const layout: Layout = new Map();
  const width = Math.max(TERMINAL.width, listSize(steps, [], ROOT).width);
  const center = width / 2;
  put(layout, START_ID, center, 0, TERMINAL);
  const end = placeList(layout, steps, [], ROOT, center, TERMINAL.height + GAP_Y);
  put(layout, END_ID, center, end + GAP_Y, END);
  return layout;
}
