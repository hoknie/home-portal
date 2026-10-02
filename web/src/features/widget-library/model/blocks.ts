export type RawBlock = Record<string, unknown> & { kind: string };

export const BLOCK_KINDS = ["stat", "text", "markdown", "list", "table", "key-values", "progress", "badge", "button", "divider", "row", "column"] as const;

export type BlockKind = (typeof BLOCK_KINDS)[number];

export const GROUP_KINDS: readonly string[] = ["row", "column"];

export const TONED: readonly string[] = ["stat", "text", "list", "progress", "badge"];

export const SLOT_MARK = "slot";

export function emptyPlace(): RawBlock {
  return { kind: "text", text: "", [SLOT_MARK]: true };
}

export function newBlock(kind: BlockKind): RawBlock {
  switch (kind) {
    case "stat":
      return { kind, label: "", value: "{{data}}" };
    case "text":
    case "markdown":
    case "badge":
      return { kind, text: "" };
    case "list":
      return { kind, items: "{{data}}", text: "{{item}}" };
    case "table":
      return { kind, items: "{{data}}", columns: [{ header: "", value: "{{item}}" }] };
    case "key-values":
      return { kind, pairs: [{ key: "", value: "" }] };
    case "progress":
      return { kind, value: "{{data}}" };
    case "button":
      return { kind, label: "", action: { refresh: true } };
    case "row":
    case "column":
      return { kind, blocks: [emptyPlace(), emptyPlace()] };
    case "divider":
      return { kind };
  }
}

export function blocksOf(settings: Record<string, unknown>): RawBlock[] {
  const blocks = settings.blocks;
  return Array.isArray(blocks) ? (blocks.filter((block) => block !== null && typeof block === "object") as RawBlock[]) : [];
}

export function replaced<Item>(items: Item[], index: number, item: Item): Item[] {
  return items.map((existing, position) => (position === index ? item : existing));
}

export function without<Item>(items: Item[], index: number): Item[] {
  return items.filter((_, position) => position !== index);
}

export type ToneMode = "fixed" | "thresholds" | "tones";

export function toneModeOf(block: RawBlock): ToneMode {
  if (block.thresholds !== undefined) {
    return "thresholds";
  }
  return block.tones !== undefined ? "tones" : "fixed";
}

export function withToneMode(block: RawBlock, mode: ToneMode): RawBlock {
  const rest = { ...block };
  delete rest.tone;
  delete rest.thresholds;
  delete rest.tones;
  if (mode === "thresholds") {
    return { ...rest, thresholds: { warning: 75, danger: 90 } };
  }
  if (mode === "tones") {
    return { ...rest, tones: {} };
  }
  return rest;
}

export function errorsUnder(errors: { field: string; message: string }[], prefix: string): Record<string, string> {
  return Object.fromEntries(
    errors
      .filter((error) => error.field === prefix || error.field.startsWith(`${prefix}.`))
      .map((error) => [error.field === prefix ? "" : error.field.slice(prefix.length + 1), error.message]),
  );
}

export function tabOfError(field: string): "data" | "content" | "general" {
  if (field.startsWith("source") || field.startsWith("refresh_seconds")) {
    return "data";
  }
  return field.startsWith("blocks") ? "content" : "general";
}
