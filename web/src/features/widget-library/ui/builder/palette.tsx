"use client";

import { useTranslations } from "next-intl";
import type { PointerEvent } from "react";

import { BLOCK_ICONS } from "../../model/block-icons";
import { BLOCK_KINDS, type BlockKind } from "../../model/blocks";

export type PaletteProps = { onAdd: (kind: BlockKind) => void; onDragStart: (kind: BlockKind, event: PointerEvent) => void };

export function Palette({ onAdd, onDragStart }: PaletteProps) {
  const t = useTranslations();
  return (
    <div className="grid gap-2" data-palette="">
      <p className="text-xs text-muted-foreground">{t("widgetBuilder.palette.hint")}</p>
      <ul className="grid grid-cols-3 gap-1.5" aria-label={t("widgetBuilder.palette.title")}>
        {BLOCK_KINDS.map((kind) => {
          const Icon = BLOCK_ICONS[kind];
          const description = t(`layoutEditor.blocks.descriptions.${kind}`);
          return (
            <li key={kind}>
              <button
                type="button"
                data-palette-kind={kind}
                title={description}
                onClick={() => onAdd(kind)}
                onPointerDown={(event) => onDragStart(kind, event)}
                className="flex h-16 w-full cursor-grab touch-none flex-col items-center justify-center gap-1 rounded-xl border border-glass-edge bg-glass-tint px-1 text-center transition-colors outline-none select-none hover:border-primary/60 hover:bg-primary/5 focus-visible:ring-[3px] focus-visible:ring-ring/50 active:cursor-grabbing"
              >
                <Icon className="size-4 text-primary" aria-hidden />
                <span className="w-full truncate text-[11px] leading-tight font-medium">{t(`layoutEditor.blocks.kinds.${kind}`)}</span>
                <span className="sr-only">{description}</span>
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
