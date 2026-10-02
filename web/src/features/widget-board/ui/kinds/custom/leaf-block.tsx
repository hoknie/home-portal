"use client";

import { useTranslations } from "next-intl";

import type { RenderedLeaf } from "@/entities/widget";
import { cn } from "@/shared/lib/cn";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/shared/ui/kit";

import { TONE_BADGE, TONE_FILL, TONE_TEXT, percentOf } from "../../../model/tones";
import { BlockIcon } from "./block-icon";
import { MarkdownBlock } from "./markdown-block";
import { WidgetButton } from "./widget-button";

export type BlockAlign = "start" | "center" | "end";

export type LeafProps = { block: RenderedLeaf; align?: BlockAlign; widget: string; scope: "private" | "public" };

const SPREADS: Record<BlockAlign, string> = { start: "justify-between", center: "justify-center", end: "justify-end" };

const LINES: Record<BlockAlign, string> = { start: "", center: "justify-center", end: "justify-end" };

const SIZES = { small: "text-sm", normal: "text-base", large: "text-2xl font-semibold" } as const;

export function LeafBlock({ block, align = "start", widget, scope }: LeafProps) {
  const t = useTranslations("widgets.custom");
  switch (block.kind) {
    case "stat":
      return (
        <div className="grid gap-1" data-block="stat" data-tone={block.tone}>
          <p className={cn("flex items-center gap-1.5 text-xs text-muted-foreground", LINES[align])}>
            <BlockIcon name={block.icon} className="size-3.5" />
            {block.label}
          </p>
          <p className={cn("text-2xl font-semibold tabular-nums", TONE_TEXT[block.tone])}>
            {block.value}
            {block.unit ? <span className="ml-1 text-sm font-normal text-muted-foreground">{block.unit}</span> : null}
          </p>
          {block.caption ? <p className="text-xs text-muted-foreground">{block.caption}</p> : null}
        </div>
      );
    case "text":
      return (
        <p
          data-block="text"
          data-tone={block.tone}
          className={cn(SIZES[block.size], block.weight === "strong" && "font-semibold", block.muted ? "text-muted-foreground" : TONE_TEXT[block.tone], "break-words whitespace-pre-line")}
        >
          {block.text}
        </p>
      );
    case "markdown":
      return <MarkdownBlock text={block.text} />;
    case "badge":
      return (
        <span data-block="badge" data-tone={block.tone} className={cn("inline-flex w-fit items-center rounded-full border px-2.5 py-0.5 text-xs font-medium", TONE_BADGE[block.tone])}>
          {block.text}
        </span>
      );
    case "progress": {
      const percent = percentOf(block.value, block.maximum);
      return (
        <div className="grid w-full gap-1.5" data-block="progress" data-tone={block.tone}>
          {block.label ? (
            <div className={cn("flex items-baseline gap-2 text-sm", SPREADS[align])}>
              <span className="text-muted-foreground">{block.label}</span>
              <span className="font-medium tabular-nums">{t("percent", { value: Math.round(percent) })}</span>
            </div>
          ) : null}
          <div role="meter" aria-label={block.label ?? t("progress")} aria-valuenow={Math.round(percent)} aria-valuemin={0} aria-valuemax={100} className="h-2 overflow-hidden rounded-full bg-muted">
            <div className={cn("h-full rounded-full transition-[width]", TONE_FILL[block.tone])} style={{ width: `${percent}%` }} />
          </div>
          {block.caption ? <p className="text-xs text-muted-foreground">{block.caption}</p> : null}
        </div>
      );
    }
    case "key-values":
      return (
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm" data-block="key-values">
          {block.pairs.map((pair, index) => (
            <div key={`${pair.key}-${index}`} className="contents">
              <dt className="text-muted-foreground">{pair.key}</dt>
              <dd className="min-w-0 break-words">{pair.value}</dd>
            </div>
          ))}
        </dl>
      );
    case "list":
      return block.items.length === 0 ? (
        <p className="text-sm text-muted-foreground">{block.empty ?? t("empty")}</p>
      ) : (
        <ul className="grid w-full gap-1.5 text-sm" data-block="list">
          {block.items.map((item, index) => (
            <li key={index} className={cn("flex items-baseline gap-3", SPREADS[align])}>
              <span className={cn("min-w-0 truncate", TONE_TEXT[item.tone])}>{item.text}</span>
              {item.secondary ? <span className="shrink-0 text-xs text-muted-foreground tabular-nums">{item.secondary}</span> : null}
            </li>
          ))}
          {block.more > 0 ? <li className="text-xs text-muted-foreground">{t("more", { count: block.more })}</li> : null}
        </ul>
      );
    case "table":
      return block.rows.length === 0 ? (
        <p className="text-sm text-muted-foreground">{block.empty ?? t("empty")}</p>
      ) : (
        <div className="w-full overflow-x-auto" data-block="table">
          <Table>
            <TableHeader>
              <TableRow className="text-xs text-muted-foreground hover:bg-transparent">
                {block.headers.map((header, index) => (
                  <TableHead key={index} scope="col" className="h-auto px-0 py-1.5 pr-3 text-xs text-muted-foreground">
                    {header}
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {block.rows.map((row, index) => (
                <TableRow key={index} className="hover:bg-transparent">
                  {row.map((cell, column) => (
                    <TableCell key={column} className="px-0 py-1.5 pr-3">
                      {cell}
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
          {block.more > 0 ? <p className="mt-1 text-xs text-muted-foreground">{t("more", { count: block.more })}</p> : null}
        </div>
      );
    case "button":
      return scope === "public" ? null : <WidgetButton button={block} widget={widget} />;
    case "divider":
      return <hr className="border-border" data-block="divider" />;
  }
}
