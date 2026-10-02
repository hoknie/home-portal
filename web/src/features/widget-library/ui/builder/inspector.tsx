"use client";

import { ArrowDown, ArrowDownRight, ArrowUp, ArrowUpLeft, Copy, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Button, Heading, IconChoice, Input } from "@/shared/ui/kit";

import { BLOCK_ICONS } from "../../model/block-icons";
import type { BlockKind, RawBlock } from "../../model/blocks";
import { type BlockPath, type Command, childrenOf } from "../../model/block-tree";
import { BlockEditor } from "../settings-dialog/block-editor";
import { ALIGN_ICONS, VALIGN_ICONS } from "../settings-dialog/choice-icons";

export type InspectorAction = Command | "duplicate" | "delete";

export type InspectorProps = {
  slot?: boolean;
  block: RawBlock | undefined;
  path: BlockPath | null;
  parentKind?: string | null;
  errors: Record<string, string>;
  missing: string[];
  onChange: (block: RawBlock) => void;
  onAction: (action: InspectorAction) => void;
  children?: ReactNode;
};

const ACTIONS: { action: InspectorAction; icon: typeof ArrowUp }[] = [
  { action: "up", icon: ArrowUp },
  { action: "down", icon: ArrowDown },
  { action: "out", icon: ArrowUpLeft },
  { action: "into", icon: ArrowDownRight },
  { action: "duplicate", icon: Copy },
  { action: "delete", icon: Trash2 },
];

const ALIGNS = ["inherit", "start", "center", "end"] as const;

const ROW_ALIGNS = ["start", "center", "end", "stretch"] as const;

function Widths({ block, onChange }: { block: RawBlock; onChange: (block: RawBlock) => void }) {
  const t = useTranslations("widgetBuilder.inspector");
  const count = childrenOf(block).length;
  const widths = Array.isArray(block.widths) && block.widths.length === count ? (block.widths as number[]) : null;
  return (
    <fieldset className="grid gap-2">
      <legend className="text-sm font-medium">{t("widths")}</legend>
      <div className="flex flex-wrap gap-2">
        {Array.from({ length: count }, (_, index) => (
          <Input
            key={index}
            type="number"
            min={1}
            max={12}
            className="h-8 w-16"
            aria-label={t("part", { number: index + 1 })}
            placeholder="1"
            value={widths?.[index] ?? ""}
            onChange={(event) => {
              const next = Array.from({ length: count }, (_, position) => (position === index ? Number(event.target.value) || 1 : (widths?.[position] ?? 1)));
              onChange({ ...block, widths: next.every((part) => part === 1) ? undefined : next });
            }}
          />
        ))}
      </div>
    </fieldset>
  );
}

const VALIGNS = ["inherit", "start", "center", "end"] as const;

export function Inspector({ slot = false, block, path, parentKind, errors, missing, onChange, onAction, children }: InspectorProps) {
  const t = useTranslations();
  if (block === undefined || path === null || slot) {
    return (
      <div className="grid content-start gap-4" data-inspector="">
        <div className="grid gap-1 rounded-xl border border-dashed border-glass-edge p-4 text-center">
          <p className="text-sm font-medium">{slot ? t("widgetBuilder.slot.title") : t("widgetBuilder.inspector.title")}</p>
          <p className="text-xs text-muted-foreground">{slot ? t("widgetBuilder.slot.hint") : t("widgetBuilder.inspector.nothing")}</p>
        </div>
        {children}
      </div>
    );
  }
  const kind = block.kind as BlockKind;
  const Icon = BLOCK_ICONS[kind] ?? BLOCK_ICONS.text;
  const align = typeof block.align === "string" ? block.align : null;
  const valign = typeof block.valign === "string" ? block.valign : null;
  const vertical = kind === "column" || (parentKind === "row" && kind !== "divider" && kind !== "row");
  return (
    <div className="@container grid content-start gap-5" data-inspector={path.join(".")}>
      <div className="flex items-center gap-3">
        <span className="grid size-9 shrink-0 place-items-center rounded-xl bg-primary/10 text-primary">
          <Icon className="size-4" aria-hidden />
        </span>
        <div className="grid min-w-0 flex-1">
          <Heading level="group" as="h3" className="text-sm font-semibold">{t(`layoutEditor.blocks.kinds.${kind}`)}</Heading>
          <p className="truncate text-xs text-muted-foreground">{t(`layoutEditor.blocks.descriptions.${kind}`)}</p>
        </div>
      </div>
      <div className="flex flex-wrap gap-0.5 rounded-xl border border-glass-edge bg-glass-tint p-0.5" role="toolbar" aria-label={t("widgetBuilder.inspector.title")}>
        {ACTIONS.map(({ action, icon: ActionIcon }) => (
          <Button key={action} type="button" variant="ghost" size="icon" className="size-8" aria-label={t(`widgetBuilder.actions.${action}`)} title={t(`widgetBuilder.actions.${action}`)} onClick={() => onAction(action)}>
            <ActionIcon aria-hidden />
          </Button>
        ))}
      </div>
      {missing.length > 0 ? (
        <ul role="alert" className="grid gap-0.5 rounded-xl border border-status-degraded/40 bg-status-degraded/10 px-3 py-2 text-xs text-foreground">
          {missing.map((field) => (
            <li key={field}>
              {field === "blocks"
                ? t(kind === "row" ? "widgetBuilder.inspector.rowNeeds" : "widgetBuilder.inspector.columnNeeds")
                : t("widgetBuilder.inspector.needs", { field: t.has(`layoutEditor.blocks.fields.${field}` as "layoutEditor.blocks.fields.label") ? t(`layoutEditor.blocks.fields.${field}` as "layoutEditor.blocks.fields.label") : field })}
            </li>
          ))}
        </ul>
      ) : null}
      {kind === "row" ? (
        <>
          <IconChoice
            label={t("widgetBuilder.inspector.rowAlign")}
            value={(align ?? "stretch") as (typeof ROW_ALIGNS)[number]}
            options={ROW_ALIGNS.map((value) => ({ value, label: t(`widgetBuilder.rowAligns.${value}`), icon: VALIGN_ICONS[value] }))}
            onChange={(value) => onChange({ ...block, align: value === "stretch" ? undefined : value })}
          />
          <Widths block={block} onChange={onChange} />
        </>
      ) : kind !== "divider" ? (
        <IconChoice
          label={t("widgetBuilder.inspector.align")}
          value={(align ?? "inherit") as (typeof ALIGNS)[number]}
          options={ALIGNS.map((value) => ({ value, label: t(`widgetBuilder.aligns.${value}`), icon: ALIGN_ICONS[value] }))}
          onChange={(value) => onChange({ ...block, align: value === "inherit" ? undefined : value })}
        />
      ) : null}
      {vertical ? (
        <IconChoice
          label={t(kind === "column" ? "widgetBuilder.inspector.columnValign" : "widgetBuilder.inspector.valign")}
          value={(valign ?? "inherit") as (typeof VALIGNS)[number]}
          options={VALIGNS.map((value) => ({ value, label: t(`widgetBuilder.valigns.${value}`), icon: VALIGN_ICONS[value] }))}
          onChange={(value) => onChange({ ...block, valign: value === "inherit" ? undefined : value })}
        />
      ) : null}
      <BlockEditor id={`builder-${path.join("-")}`} block={block} errors={errors} onChange={onChange} />
      {children}
    </div>
  );
}
