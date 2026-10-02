"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical, Plus, Settings2, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { Section } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { Button, Input } from "@/shared/ui/primitives";

import { SECTION_GRID } from "@/shared/lib/widget-grid";

import { SECTION_KIND, SECTION_SORT_PREFIX } from "../model/drag";
import { SECTION_GRID_ATTRIBUTE } from "../model/measure";

export type SectionBlockProps = {
  section: Section;
  count: number;
  widgetIds: string[];
  ghost: { column: number; row: number; width: number; rows: number } | null;
  errors: string[];
  children: ReactNode;
  onRename: (title: string) => void;
  onRemove: () => void;
  onAdd: () => void;
  onSettings: () => void;
};

export function SectionBlock(props: SectionBlockProps) {
  const t = useTranslations("layoutEditor");
  const { section, count, widgetIds } = props;
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: `${SECTION_SORT_PREFIX}${section.id}`,
    data: { kind: SECTION_KIND, section: section.id },
  });
  const title = section.title ?? t("untitled");
  return (
    <section
      ref={setNodeRef}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      className={cn("grid gap-3 rounded-2xl border border-dashed border-glass-edge p-3", isDragging && "z-20 opacity-70")}
      data-section={section.id}
      data-section-block=""
    >
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          className="flex size-8 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-glass-tint focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
          aria-label={t("moveSection", { title })}
          {...attributes}
          {...listeners}
        >
          <GripVertical className="size-4" aria-hidden />
        </button>
        <Input
          aria-label={t("sectionTitle")}
          placeholder={t("untitled")}
          className="h-8 max-w-xs"
          value={section.title ?? ""}
          onChange={(event) => props.onRename(event.target.value)}
        />
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label={t("removeSection")}
          disabled={widgetIds.length > 0 || count <= 1}
          onClick={props.onRemove}
        >
          <Trash2 aria-hidden />
        </Button>
        <Button type="button" variant="ghost" size="icon" aria-label={t("section.settings")} title={t("section.settings")} onClick={props.onSettings}>
          <Settings2 aria-hidden />
        </Button>
        <div className="ml-auto flex items-center gap-2">
          <Button type="button" variant="outline" size="sm" onClick={props.onAdd}>
            <Plus aria-hidden />
            {t("addWidget")}
          </Button>
        </div>
      </div>
      {props.errors.map((error) => (
        <p key={error} role="alert" className="text-xs text-destructive">
          {error}
        </p>
      ))}
      <div {...{ [SECTION_GRID_ATTRIBUTE]: section.id }} className={cn(SECTION_GRID, "relative min-h-20")}>
        {props.children}
        {props.ghost ? (
          <div
            aria-hidden
            data-move-ghost=""
            className="pointer-events-none rounded-2xl border-2 border-dashed border-primary bg-primary/5"
            style={{ gridColumn: `${props.ghost.column} / span ${props.ghost.width}`, gridRow: `${props.ghost.row} / span ${props.ghost.rows}` }}
          />
        ) : null}
        {widgetIds.length === 0 ? <p className="col-span-12 self-center py-4 text-center text-sm text-muted-foreground">{t("emptySection")}</p> : null}
      </div>
    </section>
  );
}
