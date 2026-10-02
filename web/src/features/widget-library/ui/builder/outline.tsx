"use client";

import { ChevronDown, GripVertical } from "lucide-react";
import { useTranslations } from "next-intl";
import { type KeyboardEvent, type PointerEvent, useState } from "react";

import { cn } from "@/shared/lib/cn";

import { OUTLINE_ATTRIBUTE, SLOT_ATTRIBUTE, textOf } from "../../model/block-drop";
import { BLOCK_ICONS } from "../../model/block-icons";
import type { BlockKind, RawBlock } from "../../model/blocks";
import { type BlockPath, childrenOf, isGroup, isSlot } from "../../model/block-tree";

export type OutlineProps = {
  blocks: RawBlock[];
  selected: BlockPath | null;
  flagged: readonly string[];
  onSelect: (path: BlockPath) => void;
  onDragStart: (path: BlockPath, event: PointerEvent) => void;
  onKey: (event: KeyboardEvent) => void;
};

export function summaryOf(block: RawBlock): string {
  const text = [block.label, block.text, block.value, block.items].find((value) => typeof value === "string" && value.trim() !== "");
  return typeof text === "string" ? text : "";
}

export function Outline({ blocks, selected, flagged, onSelect, onDragStart, onKey }: OutlineProps) {
  const t = useTranslations();
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());
  const toggle = (key: string) => setCollapsed((current) => (current.has(key) ? new Set([...current].filter((item) => item !== key)) : new Set([...current, key])));
  const rows = (list: RawBlock[], parent: BlockPath) => (
    <ul role={parent.length === 0 ? "tree" : "group"} aria-label={parent.length === 0 ? t("widgetBuilder.outline.title") : undefined} className={cn("grid gap-0.5", parent.length > 0 && "ml-4 border-l border-glass-edge pl-2")}>
      {list.map((block, index) => {
        const path = [...parent, index];
        const key = textOf(path);
        const kind = block.kind as BlockKind;
        const Icon = BLOCK_ICONS[kind] ?? BLOCK_ICONS.text;
        const empty = parent.length > 0 && isSlot(block);
        const name = empty ? t("widgetBuilder.slot.title") : t(`layoutEditor.blocks.kinds.${kind}`);
        const group = isGroup(block);
        const open = group && !collapsed.has(key);
        const chosen = selected !== null && textOf(selected) === key;
        return (
          <li key={key} role="treeitem" aria-selected={chosen} aria-expanded={group ? open : undefined}>
            <div
              {...{ [OUTLINE_ATTRIBUTE]: key }}
              className={cn("flex items-center gap-0.5 rounded-lg pr-1 transition-colors", chosen ? "bg-primary/10 text-foreground ring-1 ring-primary/40" : "hover:bg-muted/60", flagged.includes(key) && !chosen && "ring-1 ring-status-degraded/60")}
            >
              {group ? (
                <button type="button" className="flex size-6 items-center justify-center rounded text-muted-foreground hover:text-foreground" aria-label={t(open ? "widgetBuilder.outline.collapse" : "widgetBuilder.outline.expand", { name })} onClick={() => toggle(key)}>
                  <ChevronDown className={cn("size-3.5 transition-transform", !open && "-rotate-90")} aria-hidden />
                </button>
              ) : (
                <span className="size-6" aria-hidden />
              )}
              <span
                role="presentation"
                className="flex size-6 cursor-grab touch-none items-center justify-center text-muted-foreground"
                aria-label={t("widgetBuilder.outline.drag", { name })}
                onPointerDown={(event) => onDragStart(path, event)}
              >
                <GripVertical className="size-3.5" aria-hidden />
              </span>
              <button type="button" data-outline-select={key} className="flex min-w-0 flex-1 items-center gap-1.5 py-1 text-left text-sm outline-none focus-visible:underline" onClick={() => onSelect(path)} onKeyDown={onKey}>
                <Icon className={cn("size-3.5 shrink-0", empty ? "text-muted-foreground/50" : "text-primary")} aria-hidden />
                <span className={empty ? "text-muted-foreground italic" : "font-medium"}>{name}</span>
                <span className="truncate text-xs text-muted-foreground">{summaryOf(block)}</span>
              </button>
            </div>
            {open ? rows(childrenOf(block), path) : null}
          </li>
        );
      })}
    </ul>
  );
  return (
    <div className="grid gap-2" data-outline="">
      <p className="text-xs text-muted-foreground">{t("widgetBuilder.outline.hint")}</p>
      {rows(blocks, [])}
      <div {...{ [SLOT_ATTRIBUTE]: "" }} className={blocks.length === 0 ? "rounded-lg border border-dashed border-glass-edge px-2 py-4 text-center text-xs text-muted-foreground" : "h-6"} aria-hidden={blocks.length > 0 || undefined}>
        {blocks.length === 0 ? t("widgetBuilder.outline.dropHere") : null}
      </div>
    </div>
  );
}
