import { GROUP_KINDS, type RawBlock, SLOT_MARK } from "./blocks";

export type BlockPath = number[];

export type DropTarget = { parent: BlockPath; index: number };

export type Refusal = "tooDeep" | "rowFull" | "columnFull" | "intoItself";

export type Command = "up" | "down" | "out" | "into";

export type Missing = { path: BlockPath; field: string };

export const DEEPEST_GROUPS = 3;

export const MOST_IN = { row: 4, column: 12 } as const;

export const FEWEST_IN = { row: 2, column: 1 } as const;

export function isGroup(block: RawBlock | undefined): boolean {
  return block !== undefined && GROUP_KINDS.includes(block.kind);
}

export function isSlot(block: RawBlock | undefined): boolean {
  return block !== undefined && block[SLOT_MARK] === true;
}

export function slotPaths(blocks: RawBlock[], parent: BlockPath = []): string[] {
  return blocks.flatMap((block, index) => {
    const path = [...parent, index];
    return isSlot(block) ? [path.join(".")] : isGroup(block) ? slotPaths(childrenOf(block), path) : [];
  });
}

export function unmarked(blocks: RawBlock[]): RawBlock[] {
  return blocks.map((block) => {
    if (isSlot(block)) {
      return { kind: "text", text: "" };
    }
    return isGroup(block) ? { ...block, blocks: unmarked(childrenOf(block)) } : block;
  });
}

export function slotAt(blocks: RawBlock[], path: BlockPath | null): boolean {
  return path !== null && path.length > 1 && isSlot(blockAt(blocks, path));
}

export function withoutSlots(blocks: RawBlock[], inGroup = false): RawBlock[] {
  return blocks
    .filter((block) => !(inGroup && isSlot(block)))
    .map((block) => (isGroup(block) ? { ...block, blocks: withoutSlots(childrenOf(block), true) } : block));
}

export function childrenOf(block: RawBlock): RawBlock[] {
  return Array.isArray(block.blocks) ? (block.blocks as RawBlock[]) : [];
}

export function samePath(left: BlockPath, right: BlockPath) {
  return left.length === right.length && left.every((step, index) => step === right[index]);
}

export function startsWith(path: BlockPath, prefix: BlockPath) {
  return prefix.length <= path.length && prefix.every((step, index) => step === path[index]);
}

export function blockAt(blocks: RawBlock[], path: BlockPath): RawBlock | undefined {
  let list = blocks;
  let found: RawBlock | undefined;
  for (const index of path) {
    found = list[index];
    if (found === undefined) {
      return undefined;
    }
    list = childrenOf(found);
  }
  return found;
}

function listAt(blocks: RawBlock[], parent: BlockPath): RawBlock[] | undefined {
  if (parent.length === 0) {
    return blocks;
  }
  const owner = blockAt(blocks, parent);
  return isGroup(owner) ? childrenOf(owner as RawBlock) : undefined;
}

function withList(blocks: RawBlock[], parent: BlockPath, change: (list: RawBlock[]) => RawBlock[]): RawBlock[] {
  if (parent.length === 0) {
    return change(blocks);
  }
  const [first, ...rest] = parent;
  return blocks.map((block, index) => (index === first ? { ...block, blocks: withList(childrenOf(block), rest, change) } : block));
}

export function updateAt(blocks: RawBlock[], path: BlockPath, block: RawBlock): RawBlock[] {
  const parent = path.slice(0, -1);
  const index = path[path.length - 1];
  return withList(blocks, parent, (list) => list.map((existing, position) => (position === index ? block : existing)));
}

export function insertAt(blocks: RawBlock[], target: DropTarget, block: RawBlock): RawBlock[] {
  return withList(blocks, target.parent, (list) => [...list.slice(0, target.index), block, ...list.slice(target.index)]);
}

export function removeAt(blocks: RawBlock[], path: BlockPath): RawBlock[] {
  const parent = path.slice(0, -1);
  const index = path[path.length - 1];
  return withList(blocks, parent, (list) => list.filter((_, position) => position !== index));
}

export function duplicateAt(blocks: RawBlock[], path: BlockPath): [RawBlock[], BlockPath] {
  const block = blockAt(blocks, path);
  if (block === undefined) {
    return [blocks, path];
  }
  const copy = structuredClone(block);
  const next = [...path.slice(0, -1), path[path.length - 1] + 1];
  return [insertAt(blocks, { parent: path.slice(0, -1), index: next[next.length - 1] }, copy), next];
}

export function groupDepthOf(block: RawBlock): number {
  if (!isGroup(block)) {
    return 0;
  }
  return 1 + Math.max(0, ...childrenOf(block).map(groupDepthOf));
}

export function refusal(blocks: RawBlock[], block: RawBlock, target: DropTarget, from: BlockPath | null = null): Refusal | null {
  if (from !== null && startsWith(target.parent, from)) {
    return "intoItself";
  }
  if (target.parent.length + groupDepthOf(block) > DEEPEST_GROUPS) {
    return "tooDeep";
  }
  const owner = target.parent.length === 0 ? undefined : blockAt(blocks, target.parent);
  if (owner === undefined) {
    return null;
  }
  const staying = from !== null && samePath(from.slice(0, -1), target.parent);
  const count = childrenOf(owner).length + (staying ? 0 : 1);
  if (owner.kind === "row" && count > MOST_IN.row) {
    return "rowFull";
  }
  return owner.kind === "column" && count > MOST_IN.column ? "columnFull" : null;
}

export function landing(from: BlockPath, target: DropTarget): BlockPath {
  const sameParent = samePath(from.slice(0, -1), target.parent);
  const shifted = sameParent && from[from.length - 1] < target.index ? target.index - 1 : target.index;
  if (!sameParent && startsWith(target.parent, from.slice(0, -1)) && target.parent.length > from.length - 1) {
    const level = from.length - 1;
    const parent = [...target.parent];
    if (from[level] < parent[level]) {
      parent[level] -= 1;
    }
    return [...parent, shifted];
  }
  return [...target.parent, shifted];
}

export function moveTo(blocks: RawBlock[], from: BlockPath, target: DropTarget): [RawBlock[], BlockPath] {
  const block = blockAt(blocks, from);
  if (block === undefined || refusal(blocks, block, target, from) !== null) {
    return [blocks, from];
  }
  const to = landing(from, target);
  const without = removeAt(blocks, from);
  return [insertAt(without, { parent: to.slice(0, -1), index: to[to.length - 1] }, block), to];
}

export function targetOf(blocks: RawBlock[], path: BlockPath, command: Command): DropTarget | null {
  const parent = path.slice(0, -1);
  const index = path[path.length - 1];
  const siblings = listAt(blocks, parent) ?? [];
  switch (command) {
    case "up":
      return index > 0 ? { parent, index: index - 1 } : null;
    case "down":
      return index < siblings.length - 1 ? { parent, index: index + 2 } : null;
    case "out":
      return parent.length > 0 ? { parent: parent.slice(0, -1), index: parent[parent.length - 1] + 1 } : null;
    case "into": {
      const after = siblings.findIndex((sibling, position) => position > index && isGroup(sibling));
      const before = siblings.findLastIndex((sibling, position) => position < index && isGroup(sibling));
      const group = after >= 0 ? after : before;
      if (group < 0) {
        return null;
      }
      return { parent: [...parent, group], index: after >= 0 ? 0 : childrenOf(siblings[group]).length };
    }
  }
}

export function stepped(blocks: RawBlock[], path: BlockPath, command: Command): [RawBlock[], BlockPath] | Refusal | null {
  const target = targetOf(blocks, path, command);
  const block = blockAt(blocks, path);
  if (target === null || block === undefined) {
    return null;
  }
  const refused = refusal(blocks, block, target, path);
  return refused ?? moveTo(blocks, path, target);
}

const REQUIRED: Record<string, string[]> = {
  stat: ["label", "value"],
  text: ["text"],
  markdown: ["text"],
  badge: ["text"],
  list: ["items", "text"],
  table: ["items"],
  progress: ["value"],
  button: ["label"],
};

function empty(value: unknown) {
  return value === undefined || value === null || (typeof value === "string" && value.trim() === "");
}

export function missingIn(blocks: RawBlock[], parent: BlockPath = []): Missing[] {
  return blocks.flatMap((block, index) => {
    const path = [...parent, index];
    if (parent.length > 0 && isSlot(block)) {
      return [];
    }
    const own = (REQUIRED[block.kind] ?? []).filter((field) => empty(block[field])).map((field) => ({ path, field }));
    if (block.kind === "table" && (!Array.isArray(block.columns) || block.columns.length === 0)) {
      own.push({ path, field: "columns" });
    }
    if (isGroup(block)) {
      const fewest = FEWEST_IN[block.kind as keyof typeof FEWEST_IN];
      if (childrenOf(block).filter((child) => !isSlot(child)).length < fewest) {
        own.push({ path, field: "blocks" });
      }
      return [...own, ...missingIn(childrenOf(block), path)];
    }
    return own;
  });
}

export function pathOfField(field: string): [BlockPath, string] | null {
  const match = /^blocks((?:\[\d+\](?:\.blocks)?)+?)(?:\.(.+))?$/.exec(field);
  if (match === null) {
    return null;
  }
  const indices = [...match[1].matchAll(/\[(\d+)\]/g)].map((found) => Number(found[1]));
  return [indices, match[2] ?? ""];
}

export function fieldOfPath(path: BlockPath): string {
  return `blocks${path.map((index) => `[${index}]`).join(".blocks")}`;
}
