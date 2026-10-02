"use client";

import { Plus } from "lucide-react";
import { useTranslations } from "next-intl";
import type { CSSProperties } from "react";

import type { CustomWidgetData, RenderedLeaf, RenderedPart } from "@/entities/widget";
import { cn } from "@/shared/lib/cn";
import { useFrameAlign } from "@/shared/ui/widget-frame";

import { type BlockAlign, LeafBlock } from "./leaf-block";

export type Marking = { selected: string | null; flagged: readonly string[]; slots?: readonly string[] };

export type CustomWidgetProps = { data: CustomWidgetData; widget: string; scope: "private" | "public"; marking?: Marking };

type Context = { widget: string; scope: "private" | "public"; fallback: BlockAlign; marking: Marking | null };

const MARKS = "relative -m-1 rounded-lg p-1 transition-[box-shadow,background-color] [&:hover:not(:has([data-block-path]:hover))]:bg-primary/5";

function marks(context: Context, path: string) {
  if (context.marking === null) {
    return {};
  }
  const selected = context.marking.selected === path;
  const flagged = context.marking.flagged.includes(path);
  return {
    "data-block-path": path,
    "data-selected": selected || undefined,
    "data-flagged": flagged || undefined,
    className: cn(
      MARKS,
      "cursor-pointer",
      flagged && !selected && "bg-status-degraded/8 ring-1 ring-status-degraded ring-inset",
      selected && "bg-primary/8 ring-2 ring-primary ring-inset hover:bg-primary/8",
    ),
  };
}

const GAPS = { small: "gap-2", normal: "gap-4" } as const;

const PLACES: Record<BlockAlign, string> = {
  start: "items-start text-left",
  center: "items-center text-center",
  end: "items-end text-right",
};

const SELF = { start: "sm:self-start", center: "sm:self-center", end: "sm:self-end" } as const;

const CONTENT = { start: "content-start", center: "h-full content-center", end: "h-full content-end" } as const;

const ROW_ALIGNS = { start: "sm:items-start", center: "sm:items-center", end: "sm:items-end", stretch: "sm:items-stretch" } as const;

const ROW_CLASSES = "sm:[grid-template-columns:var(--row-columns)]";

function rowStyle(widths: number[] | null): CSSProperties | undefined {
  return widths === null ? undefined : ({ "--row-columns": widths.map((part) => `minmax(0,${part}fr)`).join(" ") } as unknown as CSSProperties);
}

function EmptySlot({ marked }: { marked: ReturnType<typeof marks> }) {
  const t = useTranslations("widgets.custom");
  return (
    <div {...marked} data-slot="" className={cn(marked.className, "flex min-h-12 items-center justify-center gap-1.5 border border-dashed border-glass-edge text-xs text-muted-foreground")}>
      <Plus className="size-3.5" aria-hidden />
      {t("slot")}
    </div>
  );
}

function Part({ block, context, path }: { block: RenderedPart; context: Context; path: string }) {
  const marked = marks(context, path);
  const child = (index: number) => (path === "" ? String(index) : `${path}.${index}`);
  if (block.kind === "row") {
    return (
      <div
        {...marked}
        data-block="row"
        data-align={block.align}
        style={rowStyle(block.widths)}
        className={cn("grid min-w-0", GAPS[block.gap], ROW_ALIGNS[block.align], block.widths === null ? "sm:auto-cols-fr sm:grid-flow-col" : ROW_CLASSES, marked.className)}
      >
        {block.blocks.map((inner, index) => (
          <Part key={index} block={inner} context={context} path={child(index)} />
        ))}
      </div>
    );
  }
  if (block.kind === "column") {
    return (
      <div {...marked} data-block="column" data-valign={block.valign ?? undefined} className={cn("grid min-w-0", CONTENT[block.valign ?? "start"], GAPS[block.gap], marked.className)}>
        {block.blocks.map((inner, index) => (
          <Part key={index} block={inner} context={context} path={child(index)} />
        ))}
      </div>
    );
  }
  const leaf = block as RenderedLeaf;
  const align = ("align" in leaf ? leaf.align : null) ?? context.fallback;
  if (context.marking?.slots?.includes(path)) {
    return <EmptySlot marked={marked} />;
  }
  return (
    <div {...marked} data-align={align} data-valign={leaf.kind === "divider" ? undefined : (leaf.valign ?? undefined)} className={cn("flex min-w-0 flex-col", PLACES[align], leaf.kind !== "divider" && leaf.valign ? SELF[leaf.valign] : null, marked.className)}>
      <LeafBlock block={leaf} align={align} widget={context.widget} scope={context.scope} />
    </div>
  );
}

export function CustomWidget({ data, widget, scope, marking }: CustomWidgetProps) {
  const fallback = useFrameAlign();
  const context = { widget, scope, fallback, marking: marking ?? null };
  return (
    <div className="grid w-full gap-4 self-stretch" data-custom-widget={widget}>
      {data.blocks.map((block, index) => (
        <Part key={index} block={block} context={context} path={String(index)} />
      ))}
    </div>
  );
}

export function CustomWidgetKind({ data, scope, id }: { data: CustomWidgetData; scope: "private" | "public"; id: string | null }) {
  return <CustomWidget data={data} scope={scope} widget={id ?? ""} />;
}
