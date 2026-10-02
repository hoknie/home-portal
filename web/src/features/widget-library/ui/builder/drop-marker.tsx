"use client";

import { useTranslations } from "next-intl";

import type { Drop } from "../../model/block-drop";
import { BLOCK_ICONS } from "../../model/block-icons";
import type { BlockKind } from "../../model/blocks";
import type { Ghost } from "../../model/use-block-drag";

export function DropMarker({ drop, ghost }: { drop: Drop | null; ghost: Ghost | null }) {
  const t = useTranslations("layoutEditor.blocks.kinds");
  const kind = ghost?.block.kind as BlockKind | undefined;
  const Icon = kind ? (BLOCK_ICONS[kind] ?? BLOCK_ICONS.text) : null;
  const box = drop?.box;
  const style =
    !drop || !box
      ? undefined
      : drop.mode === "box"
        ? { top: box.top, left: box.left, width: box.width, height: box.height }
        : drop.mode === "line-x"
          ? { top: box.top, left: box.left - 2, width: 3, height: box.height }
          : { top: box.top - 2, left: box.left, width: box.width, height: 3 };
  return (
    <>
      {drop ? (
        <div
          aria-hidden
          data-drop-marker={drop.mode}
          className={drop.mode === "box" ? "pointer-events-none fixed z-50 rounded-lg border-2 border-dashed border-primary/70 bg-primary/10" : "pointer-events-none fixed z-50 rounded-full bg-primary shadow-[0_0_0_3px] shadow-primary/20"}
          style={style}
        />
      ) : null}
      {ghost && kind && Icon ? (
        <div
          aria-hidden
          data-drag-ghost=""
          className="glass-panel pointer-events-none fixed z-50 flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium"
          style={{ top: ghost.y + 12, left: ghost.x + 12 }}
        >
          <Icon className="size-3.5 text-primary" aria-hidden />
          {t(kind)}
        </div>
      ) : null}
    </>
  );
}
